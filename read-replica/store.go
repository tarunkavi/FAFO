package main

import (
	"database/sql"
)

type DB struct {
	primary *sql.DB
	replica *sql.DB
}

func NewDB() *DB {
	return &DB{
		primary: connectDB(primaryHost, primaryPort, dbUser, dbPassword, dbName),
		replica: connectDB(replicaHost, replicaPort, dbUser, dbPassword, dbName),
	}
}

func (d *DB) Close() {
	d.primary.Close()
	d.replica.Close()
}

func (d *DB) Exec(query string, args ...any) (sql.Result, error) {
	return d.primary.Exec(query, args...)
}

func (d *DB) Query(query string, args ...any) (*sql.Rows, error) {
	return d.replica.Query(query, args...)
}

func (d *DB) QueryRow(query string, args ...any) *sql.Row {
	return d.replica.QueryRow(query, args...)
}

func (d *DB) QueryPrimary(query string, args ...any) (*sql.Rows, error) {
	return d.primary.Query(query, args...)
}

func (d *DB) Init() error {
	_, err := d.primary.Exec(`
		CREATE TABLE IF NOT EXISTS poc_test (
			id      SERIAL PRIMARY KEY,
			label   TEXT        NOT NULL,
			payload TEXT,
			ts      TIMESTAMPTZ NOT NULL DEFAULT now()
		)`)
	return err
}

// LagStats holds replication lag observed from both sides.
type LagStats struct {
	// From pg_stat_replication on the PRIMARY (one entry per connected standby).
	Standbys []StandbyLag `json:"standbys"`
	// From the REPLICA: how many seconds behind the last replayed transaction is.
	ReplicaReplayLagSec *float64 `json:"replica_replay_lag_sec"`
}

// StandbyLag is a single row from pg_stat_replication.
type StandbyLag struct {
	ClientAddr string  `json:"client_addr"`
	State      string  `json:"state"`
	WriteLag   *string `json:"write_lag"`
	FlushLag   *string `json:"flush_lag"`
	ReplayLag  *string `json:"replay_lag"`
}

// LagStats queries replication lag from both the primary and the replica.
//
//   - Primary side : pg_stat_replication  (write / flush / replay WAL lag per standby)
//   - Replica side : now() - pg_last_xact_replay_timestamp()  (wall-clock seconds behind)
func (d *DB) LagStats() (*LagStats, error) {
	stats := &LagStats{}

	// 1. WAL lag from primary
	rows, err := d.primary.Query(`
		SELECT
			coalesce(client_addr::text, 'unknown'),
			state,
			write_lag::text,
			flush_lag::text,
			replay_lag::text
		FROM pg_stat_replication`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	for rows.Next() {
		var s StandbyLag
		if err := rows.Scan(&s.ClientAddr, &s.State, &s.WriteLag, &s.FlushLag, &s.ReplayLag); err != nil {
			return nil, err
		}
		stats.Standbys = append(stats.Standbys, s)
	}

	// 2. Wall-clock replay lag from replica
	// pg_last_xact_replay_timestamp() returns NULL when no transaction has been
	// replayed yet (e.g. replica just started), so we handle that gracefully.
	var lagSec sql.NullFloat64
	err = d.replica.QueryRow(`
		SELECT EXTRACT(EPOCH FROM (now() - pg_last_xact_replay_timestamp()))
	`).Scan(&lagSec)
	if err != nil {
		return nil, err
	}
	if lagSec.Valid {
		v := lagSec.Float64
		stats.ReplicaReplayLagSec = &v
	}

	return stats, nil
}

type Record struct {
	ID      int    `json:"id"`
	Label   string `json:"label"`
	Payload string `json:"payload"`
	Ts      string `json:"ts"`
}

func (d *DB) Write(label, payload string) (int, error) {
	var id int
	err := d.primary.QueryRow(
		"INSERT INTO poc_test (label, payload) VALUES ($1, $2) RETURNING id",
		label, payload,
	).Scan(&id)
	return id, err
}

func (d *DB) ReadByID(id int) (*Record, bool, error) {
	var r Record
	err := d.replica.QueryRow(
		"SELECT id, label, payload, ts FROM poc_test WHERE id = $1", id,
	).Scan(&r.ID, &r.Label, &r.Payload, &r.Ts)
	if err == sql.ErrNoRows {
		return nil, false, nil
	}
	if err != nil {
		return nil, false, err
	}
	return &r, true, nil
}

func (d *DB) Read() ([]Record, error) {
	rows, err := d.replica.Query(
		"SELECT id, label, payload, ts FROM poc_test ORDER BY id",
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var records []Record
	for rows.Next() {
		var r Record
		if err := rows.Scan(&r.ID, &r.Label, &r.Payload, &r.Ts); err != nil {
			return nil, err
		}
		records = append(records, r)
	}
	return records, rows.Err()
}

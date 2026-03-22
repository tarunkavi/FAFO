package main

import (
	"database/sql"
	"fmt"
	"log"

	_ "github.com/lib/pq"
)

const (
	primaryHost = "localhost"
	primaryPort = 5432
	replicaHost = "localhost"
	replicaPort = 5433
	dbUser      = "postgres"
	dbPassword  = "demo123"
	dbName      = "testdb"
)

// connectDB opens a *sql.DB using a postgres DSN and verifies the connection.
func connectDB(host string, port int, user, password, dbname string) *sql.DB {
	dsn := fmt.Sprintf(
		"host=%s port=%d user=%s password=%s dbname=%s sslmode=disable",
		host, port, user, password, dbname,
	)
	db, err := sql.Open("postgres", dsn)
	if err != nil {
		log.Fatalf("sql.Open failed (%s:%d): %v", host, port, err)
	}
	if err = db.Ping(); err != nil {
		log.Fatalf("db.Ping failed (%s:%d): %v", host, port, err)
	}
	return db
}

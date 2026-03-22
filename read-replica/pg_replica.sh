echo "Postgres Read Replica Setup using Podman...."

wait_for_postgress(){
    local container_name=$1
    local max_attempts=30
    local attempt=1
    
    echo " Waiting for $container_name to be ready..."
    while [ $attempt -le $max_attempts ]; do
        if podman exec $container_name pg_isready -U postgres >/dev/null 2>&1; then
            echo "$container_name is ready!"
            return 0
        fi
        echo "   Attempt $attempt/$max_attempts..."
        sleep 2
        attempt=$((attempt + 1))
    done
    
    echo " Error: $container_name failed to start"
    podman logs $container_name
    exit 1

}


echo "Cleaning up any existing setup..."
podman rm -f pg-primary pg-replica 2>/dev/null || true
podman network rm pgnet 2>/dev/null || true
podman volume rm pgdata-primary pgdata-replica 2>/dev/null || true

echo "Creating network and storage volumes..."
podman network create pgnet
podman volume create pgdata-primary
podman volume create pgdata-replica

echo " Starting primary PostgreSQL container..."
podman run -d --name pg-primary --net pgnet -p 5432:5432 \
    -e POSTGRES_PASSWORD=demo123 \
    -e POSTGRES_USER=postgres \
    -e POSTGRES_DB=testdb \
    -v pgdata-primary:/var/lib/postgresql/data \
    docker.io/library/postgres:15 \
    -c wal_level=replica \
    -c max_wal_senders=3 \
    -c hot_standby=on

# wal_level = Tells PostgreSQL to write detailed WAL logs suitable for replication
# max_wal_senders = Controls how many replicas can connect simultaneously to stream WAL
# hot_standby = Allows replicas to serve read queries

wait_for_postgress pg-primary

echo " Configuring primary for replication..."
podman exec pg-primary psql -U postgres -c "CREATE USER repluser REPLICATION LOGIN PASSWORD 'replica123';"
podman exec pg-primary bash -c "echo 'host replication repluser all md5' >> /var/lib/postgresql/data/pg_hba.conf"
podman exec pg-primary psql -U postgres -c "SELECT pg_reload_conf();"

# Create base backup and create the replicas
echo " Creating base backup for replica..."
podman run --rm --net pgnet \
    -v pgdata-replica:/backup \
    -e PGPASSWORD=replica123 \
    docker.io/library/postgres:15 \
    pg_basebackup -h pg-primary -U repluser -D /backup -v -P

# Configure replica settings
echo "🔧 Configuring replica settings..."
podman run --rm -v pgdata-replica:/data alpine sh -c "
cat > /data/postgresql.auto.conf << 'EOF'
# POC Replica Configuration
primary_conninfo = 'host=pg-primary user=repluser password=replica123'
hot_standby = on
EOF

# Create standby signal file
touch /data/standby.signal
"

# Start replica container
echo " Starting replica PostgreSQL container..."
podman run -d --name pg-replica --net pgnet -p 5433:5432 \
    -v pgdata-replica:/var/lib/postgresql/data \
    docker.io/library/postgres:15

# Wait for replica to be ready
wait_for_postgres pg-replica

echo ""
echo " POC Setup Complete!"
echo "================================================"
echo "Connection Details:"
echo "   Primary:  localhost:5432 (postgres/demo123) - Read/Write"
echo "   Replica:  localhost:5433 (postgres/demo123) - Read Only"
echo ""
echo "   Quick Test Commands:"
echo "   Connect to primary: podman exec -it pg-primary psql -U postgres"
echo "   Connect to replica: podman exec -it pg-replica psql -U postgres"
echo ""
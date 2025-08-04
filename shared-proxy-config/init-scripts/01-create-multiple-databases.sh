#!/bin/bash
# Script to create multiple databases in PostgreSQL
# Used by the shared proxy to initialize databases for different services

set -e
set -u

function create_user_and_database() {
    local database=$1
    local user=${2:-$database}
    local password=${3:-${database}pass}
    
    echo "Creating user and database '$database'"
    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
        -- Create user if not exists
        DO
        \$\$
        BEGIN
            IF NOT EXISTS (
                SELECT FROM pg_catalog.pg_user
                WHERE usename = '$user'
            ) THEN
                CREATE USER $user WITH PASSWORD '$password';
            END IF;
        END
        \$\$;

        -- Create database if not exists
        SELECT 'CREATE DATABASE $database OWNER $user'
        WHERE NOT EXISTS (
            SELECT FROM pg_database 
            WHERE datname = '$database'
        )\gexec

        -- Grant privileges
        GRANT ALL PRIVILEGES ON DATABASE $database TO $user;
EOSQL
}

# Main postgres user already exists, created by the Docker image
echo "Multiple database creation script started"

# Parse POSTGRES_MULTIPLE_DATABASES variable
if [ -n "${POSTGRES_MULTIPLE_DATABASES:-}" ]; then
    echo "Creating multiple databases: $POSTGRES_MULTIPLE_DATABASES"
    
    # Split by comma and create each database
    IFS=',' read -ra DATABASES <<< "$POSTGRES_MULTIPLE_DATABASES"
    for db in "${DATABASES[@]}"; do
        db=$(echo "$db" | tr -d ' ')  # Remove spaces
        if [ -n "$db" ]; then
            create_user_and_database "$db"
        fi
    done
fi

# Create any additional users or databases with custom passwords
# You can add specific database setups here
# Example:
# create_user_and_database "special_db" "special_user" "special_password"

# Create feedback database
create_user_and_database "feedback" "feedback" "feedbackpass"

echo "Multiple database creation completed"
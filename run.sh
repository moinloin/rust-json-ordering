#!/bin/bash

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

print_header() {
    echo -e "\n${BLUE}══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}   ${1}${NC}"
    echo -e "${BLUE}══════════════════════════════════════════════════════════════${NC}\n"
}

print_step() {
    echo -e "${GREEN}➤ ${1}${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ ${1}${NC}"
}

print_error() {
    echo -e "${RED}✘ ${1}${NC}"
}

print_success() {
    echo -e "${GREEN}✓ ${1}${NC}"
}

print_header "JSON Order Preservation Test"

if [ ! -f "json.txt" ]; then
    print_warning "json.txt not found in current directory. Creating a sample file..."
    cat > json.txt << EOF
{
    "movies": [
        {
            "title": "Inception",
            "director": "Christopher Nolan",
            "year": 2010,
            "genre": "Sci-Fi",
            "locations": ["Cinema City Berlin", "Movieplex Hamburg"]
        },
        {
            "title": "The Grand Budapest Hotel",
            "director": "Wes Anderson",
            "year": 2014,
            "genre": "Comedy",
            "locations": ["Filmtheater München", "Kino Köln"]
        }
    ]
}
EOF
    print_success "Created sample json.txt file"
fi

print_step "Stopping any existing containers..."
docker compose down -v --remove-orphans >/dev/null 2>&1

print_step "Starting containers in the background..."
if ! docker compose up -d; then
    print_error "Failed to start containers!"
    exit 1
fi

print_step "Waiting for PostgreSQL to initialize..."
for i in {1..15}; do
    echo -ne "${YELLOW}${i}...${NC} "
    sleep 1
done
echo ""

print_step "Copying json.txt into the container..."
container_id=$(docker compose ps -q rust-app)
if [ -z "$container_id" ]; then
    print_warning "Rust app container not running yet, will copy file when running"
else
    if ! docker cp json.txt "$container_id":/app/json.txt; then
        print_warning "Could not copy file to container, will fall back to volume mount"
    else
        print_success "File copied successfully"
    fi
fi

print_header "Running Test Application"

print_step "Executing the Rust application..."
docker compose run --rm rust-app

print_header "Database Query Results"

print_step "Querying PostgreSQL database directly:"
docker compose exec postgres psql -U testuser -d testdb -c "SELECT id, jsonb_pretty(data_jsonb) AS formatted_jsonb, raw_text FROM json_test;"

print_header "Test Complete"

print_step "Stopping containers..."
docker compose down --remove-orphans >/dev/null 2>&1
print_success "All done! Check the results above."
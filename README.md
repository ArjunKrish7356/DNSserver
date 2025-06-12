# DNS Resolver (asyncDnsResolver)

A high-performance, asynchronous DNS resolver implemented in Rust using Tokio for concurrent request handling.

## Overview

This DNS resolver implements a recursive DNS resolution algorithm that queries authoritative name servers to resolve domain names to IP addresses. The resolver starts from root name servers and follows the DNS hierarchy to find the authoritative answer for any given domain.

## How It Works

### DNS Resolution Theory

The Domain Name System (DNS) is a hierarchical distributed naming system that translates human-readable domain names (like `example.com`) into IP addresses (like `192.0.2.1`). Our resolver implements the recursive resolution process:

1. **Query Reception**: The resolver listens on port 2053 for incoming DNS queries from clients
2. **Recursive Resolution**: Starting from root name servers, the resolver follows the DNS hierarchy:
   - Queries root name servers (starting with `192.203.230.10`)
   - Follows referrals to authoritative name servers
   - Continues until it finds the final answer
3. **Response**: Returns the resolved IP address to the client

### Architecture

The resolver uses an asynchronous, event-driven architecture built on Tokio:

- **Concurrent Request Handling**: Each incoming request is handled in a separate async task
- **Non-blocking I/O**: UDP socket operations are asynchronous, allowing high throughput
- **Recursive Algorithm**: Implements iterative queries to name servers following DNS referrals

### Key Components

1. **DNS Packet Parser**: Custom implementation for parsing and constructing DNS packets
2. **Recursive Resolver**: Core algorithm that follows DNS hierarchy to resolve queries
3. **Name Server Fetcher**: Handles communication with authoritative name servers
4. **Async Runtime**: Tokio-based async runtime for concurrent request processing

## Features

- ✅ Asynchronous request handling using Tokio
- ✅ Recursive DNS resolution following RFC standards
- ✅ Support for A record queries (IPv4 addresses)
- ✅ Root name server bootstrapping
- ✅ Error handling and recovery
- ✅ Concurrent client support
- ✅ Custom DNS packet parsing and construction

## Prerequisites

- Rust 1.75 or later
- Cargo package manager

## Installation & Setup

### Building the Project

```bash
# Clone or navigate to the project directory
cd DNSserver

# Build the release version
cargo build --release --bin asyncDnsResolver
```

### Running the DNS Resolver

```bash
# Run the built binary
./target/release/asyncDnsResolver
```

The resolver will start listening on:
- **Port 2053**: For incoming DNS queries from clients
- **Random port**: For outgoing queries to name servers

You should see output similar to:
```
Listening on 127.0.0.1:2053
Bound to 0.0.0.0:43521
```

## Usage

### Testing the Resolver

You can test the DNS resolver using standard DNS tools:

```bash
# Using dig
dig @127.0.0.1 -p 2053 example.com

# Using nslookup
nslookup example.com 127.0.0.1 2053
```

### Docker Deployment

The project includes a Dockerfile for containerized deployment:

```bash
# Build the Docker image
docker build -t dns-resolver .

# Run the container
docker run -p 53:53/udp -p 2053:2053/udp dns-resolver
```

## Configuration

The resolver currently uses these default settings:
- **Listen Address**: `127.0.0.1:2053`
- **Root Name Server**: `192.203.230.10` (a.root-servers.net)
- **Maximum Resolution Depth**: 13 iterations (prevents infinite loops)
- **Buffer Size**: 4096 bytes for DNS packets

## Technical Details

### DNS Packet Structure

The resolver implements custom parsing for standard DNS packet format:
- Header (12 bytes): Contains flags, question count, answer count, etc.
- Questions: Domain name queries
- Answers: Resolved records
- Authority: Authoritative name server records
- Additional: Additional helpful records

### Resolution Algorithm

1. Parse incoming DNS query packet
2. Extract the domain name from the question section
3. Start with a root name server IP
4. Iteratively query name servers:
   - Send query to current name server
   - Parse response for either answer or referral
   - Follow referrals to more specific name servers
   - Stop when authoritative answer is found
5. Construct and send response packet back to client

### Error Handling

The resolver includes comprehensive error handling for:
- Network connectivity issues
- Malformed DNS packets
- Missing name servers
- Query timeouts
- Buffer overflow protection

## Performance

- **Concurrent Processing**: Multiple queries handled simultaneously
- **Async I/O**: Non-blocking network operations
- **Memory Efficient**: Fixed-size buffers with bounds checking
- **Fast Startup**: Minimal dependencies and quick initialization

## Limitations

Current implementation focuses on:
- A record queries (IPv4 addresses) only
- Basic recursive resolution
- UDP transport only

Future enhancements could include:
- AAAA records (IPv6)
- CNAME resolution
- DNS caching
- TCP fallback for large responses
- DNSSEC validation

## Contributing

To contribute to this project:
1. Ensure Rust 1.75+ is installed
2. Run tests: `cargo test`
3. Format code: `cargo fmt`
4. Check for issues: `cargo clippy`

## License

This project is available under standard open source licenses.

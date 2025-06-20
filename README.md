# Hyli DeFi App - ZKHack Berlin 2025

This repository provides a scaffold to build DeFi applications on the Hyli network with support for both Risc0 and Noir contracts. This project integrates ZKPassport for identity verification with an AMM (Automated Market Maker) built on Hyli.

## Architecture

The application follows a client-server model with proof composition capabilities:

- **Frontend**: React-based UI with wallet integration and ZKPassport SDK
- **Backend Server**: Handles transaction creation, proving, and submission
- **Smart Contracts**: 
  - Noir contracts for identity verification (ZKPassport integration)
  - Risc0 contracts for stateful AMM logic
- **Proof Composition**: Combines multiple proof types in atomic transactions

## Prerequisites

To get started, you need to install the following tools:

### Core Dependencies
- **Docker** (with docker-compose) - [Install Docker](https://docs.docker.com/compose/install/)
- **Rust** - [Install Rust](https://rustup.rs/)
- **RISC Zero zkVM** - [Install RISC Zero](https://dev.risczero.com/api/zkvm/install)
- **Bun** (or npm/yarn) - [Install Bun](https://bun.sh/)

### Noir Toolchain
For Noir contract development, install specific versions:

```bash
# Install Noir version 1.0.0-beta.3
noirup -v 1.0.0-beta.3

# Install Barretenberg (BB) version 0.82.2
bbup -v 0.82.2
```

> **Note**: If `bbup` command is not found, you may need to install it first:
> ```bash
> curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
> source ~/.bashrc
> bbup -v 0.82.2
> ```

## Quick Start

### 1. Clone and Setup

```bash
git clone <this-repository>
cd hyli-defi-app
```

### 2. Start the Hyli Node

Launch the development node and wallet services:

```bash
docker-compose up -d
```

This will:
- Start a local Hyli node in development mode
- Launch the wallet server and UI
- Set up the necessary development environment

### 3. Start the Backend Server

From the root directory:

```bash
RISC0_DEV_MODE=true cargo run -p server
```

> **Important**: The `RISC0_DEV_MODE=true` flag is required for development to avoid lengthy proof generation times.

### 4. Start the Frontend

Navigate to the frontend and start the development server:

```bash
cd front
bun install
bun run dev
```

### 5. Access the Application

- **Frontend**: http://localhost:5173 (default Vite port)
- **Hyli Explorer**: Use the [official Hyli explorer](https://explorer.hyli.org) and switch network to `localhost`
- **Test Account**: Use username `hyli` with password `hylisecure` for testing

## Development

### Contract Development

This project supports two types of contracts:

#### Risc0 Contracts (Stateful)
- Located in `contracts/contract1/` and `contracts/contract2/`
- Written in Rust using the Risc0 zkVM
- Handle stateful AMM logic, token transfers, and complex computations
- Automatically rebuilt when changes are made

#### Noir Contracts (Privacy-Focused)
- Used for identity verification and private computations
- Stateless circuits ideal for ZKPassport integration
- Combined with Risc0 contracts via proof composition

### Building Contracts

For reproducible builds:
```bash
cargo build -p contracts --features build --features all
```

For development/testing (non-reproducible):
```bash
cargo build -p contracts --features build --features all --features nonreproducible
```

### Making Contract Changes

When modifying contracts, you may encounter program ID mismatches. To resolve:

```bash
# Clean state and restart
rm -rf ./data
docker-compose down --volumes --remove-orphans
docker-compose up -d
```

### Frontend Integration

The frontend uses the `hyli-wallet` package for wallet integration:

```bash
cd front
bun install hyli-wallet
```

Key wallet integration patterns:
- `WalletProvider` for wallet context
- `useWallet()` hook for wallet operations
- `createIdentityBlobs()` for transaction signing

## Project Structure

```
hyli-defi-app/
├── contracts/           # Smart contracts
│   ├── contract1/       # AMM contract (Risc0)
│   ├── contract2/       # Token contract (Risc0)
│   └── metadata.rs      # Contract metadata
├── server/              # Backend API server
├── front/               # React frontend
├── docs/                # Documentation
│   ├── ImplementationPlan.md
│   └── LLM-Hyli-ZKPassport-Boundless.md
├── docker-compose.yml   # Development environment
└── README.md
```

## Troubleshooting

### Common Issues

1. **`bbup` command not found**:
   ```bash
   curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
   source ~/.bashrc
   ```

2. **Program ID mismatch**: Clean data and restart services
3. **Proof generation timeout**: Ensure `RISC0_DEV_MODE=true` is set
4. **Docker issues**: Verify Docker and docker-compose are properly installed

### Getting Help

- Join the [Hyli Builder Group](https://docs.hyli.org/) for community support
- Check the [ZKHack Berlin documentation](https://docs.hyli.org/zkhack/)
- Visit us IRL during ZKHack Berlin for debugging help!

## Resources

- [Hyli Documentation](https://docs.hyli.org)
- [ZKHack Berlin Quickstart](https://docs.hyli.org/zkhack/)
- [Risc0 Documentation](https://dev.risczero.com/)
- [Noir Documentation](https://noir-lang.org/)
- [ZKPassport SDK](https://zkpassport.xyz)

## Examples

For additional examples and inspiration:
- [Hyli Wallet Implementation](https://github.com/hyli-org/wallet) - Noir + Risc0 integration
- [EZ Casino](https://github.com/hyli-org/ezcasino) - Blackjack game example
- [More examples available in Hyli documentation](https://docs.hyli.org)

---

Built for ZKHack Berlin 2025 🚀

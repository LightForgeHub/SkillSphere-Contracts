# SkillSphere Smart Contracts

Welcome to the SkillSphere smart contracts package! This is where all the blockchain magic happens using Stellar's powerful Soroban smart contract platform. Our contracts power a decentralized knowledge marketplace, creating a peer-to-peer consulting economy where experts and knowledge seekers connect directly without intermediaries. 🚀

## 📖 Table of Contents

1. **🌟 About The Project**
2. **🔧 Prerequisites**
3. **⚙️ Environment Setup**
4. **💰 Wallet Configuration**
5. **🛠️ Build & Deployment**
6. **✅ Testing & Execution**
7. **🏗️ Contract Architecture**
8. **📌 Example Usage**
9. **🤝 Contributing**
10. **💡 Tips**
11. **❓ Troubleshooting**
12. **🔗 Useful Links**

---

## 🌟 About The Project

SkillSphere is a protocol designed to democratize access to global expertise. It enables a peer-to-peer consulting economy where experts and knowledge seekers connect directly via blockchain technology.

The core innovation is **"TrustFlow"**: a system that combines on-chain identity verification with trustless, second-by-second streaming payments. This ensures that experts are compensated fairly for their time while users only pay for the exact value they receive, eliminating the need for intermediaries or large upfront fees.

### The Problem It Solves

* **Trust Friction:** Eliminates the risk of non-payment for experts and the risk of low-quality service for users via escrow and identity reputation.
* **High Fees:** Removes centralized middlemen who typically take 20-30% commissions.
* **Payment Inefficiency:** Replaces rigid hourly billing with fluid, real-time settlement ($0.01/second).

---

## 🔧 Prerequisites

Before starting, make sure you have the following dependencies installed:

### 1️⃣ Install Rust

* **For Linux/macOS:**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

* **For Windows:** Download and install Rust from **[rust-lang.org](https://rust-lang.org)**.

* **Add WebAssembly Target:**

```bash
rustup target add wasm32-unknown-unknown
```

### 2️⃣ Install Soroban CLI

* **Using Cargo:**

```bash
cargo install --locked soroban-cli
```

* **Using Homebrew (macOS, Linux):**

```bash
brew install soroban-cli
```

---

## ⚙️ Environment Setup

### 1. Clone the repository

```bash
git clone https://github.com/your-username/skillsphere-contracts.git
cd skillsphere-contracts
```

### 2. Build the contracts

```bash
soroban build
```

### 3. Run tests

```bash
cargo test
```

---

## 💰 Wallet Configuration

1. Install a Stellar wallet (e.g., **[Freighter Wallet](https://www.freighter.app/)**).
2. Create a new wallet and securely store your secret keys.
3. Connect the wallet to the Stellar testnet or Futurenet.
4. Fund your account using the **[Stellar Laboratory](https://laboratory.stellar.org/#account-creator?network=test)** friendbot.

---

## 🛠️ Build & Deployment

### 1. Compile the contracts in release mode

```bash
cargo build --release --target wasm32-unknown-unknown
```

### 2. Optimize WASM files for deployment

```bash
make optimize
```

### 3. Deploy contracts using Soroban CLI

```bash
# Deploy Identity Registry
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/identity_registry_contract.wasm \
  --network testnet

# Deploy Payment Vault
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/payment_vault_contract.wasm \
  --network testnet

# Deploy other contracts similarly...
```

---

## ✅ Testing & Execution

### Run unit tests:

```bash
cargo test
```

### Run integration tests:

```bash
cargo test --test integration_tests
```

### Interact with deployed contracts:

Use the Soroban CLI or supported wallet tools to invoke contract functions and test the full workflow.

---

## 🏗️ Contract Architecture

The system is built as a set of interacting smart contracts, each handling a specific domain of the protocol.

| Contract Module | Directory | Role & Responsibility |
| :--- | :--- | :--- |
| **Identity Registry** | `identity-registry-contract` | **Trust & Verification.** Manages expert verification status, issues Soulbound Tokens (SBTs), and maintains the ban list for bad actors. |
| **Payment Vault** | `payment-vault-contract` | **Escrow & Settlement.** Securely holds user deposits and executes streaming withdrawals based on cryptographic proofs of time. |
| **Manager Core** | `skillsphere-core-contract` | **Orchestration.** The main entry point for client applications. It coordinates the Registry and Vault to initialize sessions safely. |
| **Scheduler** | `calendar-scheduling-contract` | **Availability.** Manages expert working hours, books time slots, and enforces cancellation policies to prevent double-booking. |
| **Reputation** | `reputation-scoring-contract` | **Incentives.** Calculates and stores immutable reliability scores based on successful session completion versus disputes. |
| **Asset Factory** | `knowledge-assets-contract` | **Monetization.** Mints Knowledge NFTs (course materials) and POAPs (Proof of Attendance) for gamified learning. |

### 🚀 Key Features

* **Identity-First Trust:** Utilizes Soulbound Tokens (SBTs) to create a permanent, non-transferable reputation layer for experts.
* **Streaming Settlement:** Supports high-frequency micropayments where funds are released linearly over time.
* **Session Keys:** Implements ephemeral session signers, allowing client-side applications to auto-sign payment "ticks" securely without constant user manual approval.
* **Trustless Escrow:** Funds are locked in the `Payment Vault` during a session and can be refunded instantly if a stream is cancelled.

---

## 📌 Example Usage

For a practical example of how to interact with these contracts, check out Stellar's official documentation on **[Smart Contracts](https://developers.stellar.org/docs/smart-contracts)**.

### Sample Contract Invocation:

```bash
# Register an expert in the Identity Registry
soroban contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  -- register_expert \
  --expert <EXPERT_ADDRESS> \
  --metadata <METADATA_HASH>
```

---

## 🤝 Contributing

Contributions are welcome! Please follow these steps:

1. **Follow Rust best practices**
2. **Ensure all tests pass** before submitting
3. **Document your changes** clearly
4. **Add test cases** for new functionality
5. **Check the Issues tab** for open tasks
6. **Fork the repository** and create a feature branch (`git checkout -b feature/amazing-feature`)
7. **Commit your changes** and push to the branch
8. **Submit a Pull Request**

---

## 💡 Tips

* Use the **Soroban CLI** for local development and testing
* Test thoroughly on **testnet** before deploying to mainnet
* Keep contract size **optimized** to reduce deployment costs
* Monitor **gas usage** for complex operations
* Use **events** for tracking contract state changes
* Implement proper **error handling** in all contract functions
* Store contract IDs in environment variables for easy management

---

## ❓ Troubleshooting

If you encounter any issues, try these solutions:

* **Compilation Errors:** Ensure all dependencies are installed and updated. Run `rustup update` and `cargo update`.
* **Deployment Issues:** Verify you're connected to the correct network (testnet or mainnet) and that your wallet is properly funded.
* **Test Failures:** Check detailed error messages from `cargo test` to debug the issue. Use `RUST_LOG=debug cargo test` for verbose output.
* **WASM Build Errors:** Ensure you have the `wasm32-unknown-unknown` target installed: `rustup target add wasm32-unknown-unknown`
* **Network Connection Issues:** Verify your network configuration and RPC endpoint are correct.

For additional help, open an issue on GitHub or reach out to the maintainers.

---

## 🔗 Useful Links

* **[Soroban Documentation](https://soroban.stellar.org/docs)**
* **[Rust Documentation](https://doc.rust-lang.org/)**
* **[Stellar Documentation](https://developers.stellar.org/)**
* **[Stellar Laboratory](https://laboratory.stellar.org/)**
* **[Freighter Wallet](https://www.freighter.app/)**

---

**This README is based on Stellar's official documentation.**

**Built with ❤️ for the Stellar Community.**

🚀 **Happy coding!**
<!-- Verified for issue #31: feat(vault): implement platform revenue service fee deduction in finalize_session -->

# mycelix-supplychain

Verifiable supply-chain provenance for Mycelix. Convert ERP/IoT events into signed DKG claims and portable VCs with lineage proofs.

## Features

- **Event → VC → DKG claim** with hash-linked lineage
- **Built-in adapters** (CSV, MQTT); pluggable ERP/EDI
- **Verifier UI + CLI**; export "Product Passport" bundles
- **Idempotent ingestion**, replay-safe, tamper-evident logs
- **Selective disclosure** via SD-JWT/BBS+ cryptography

## Quickstart

```bash
# Rust service
cd rust/service
cargo run

# Post a sample event
curl -X POST http://localhost:8080/v1/events \
  -H 'content-type: application/json' \
  -d @specs/examples/batch_produced.json
```

## Architecture

1. **Ingest**: Adapters normalize inputs → `SupplyEvent` (JSON Schema)
2. **Sign**: Create VC (issuer DID, selective-disclosure)
3. **Claim**: Project VC → DKG `EpistemicClaim` + lineage (prev hashes)
4. **Publish**: Write to DKG; return claim ID + proofs
5. **Verify**: Dashboard/SDK resolve lineage and validate signatures

```
┌─────────────┐      ┌──────────────┐      ┌─────────────┐
│ ERP/IoT/CSV │─────▶│ Provenance   │─────▶│ DKG Network │
│   Sources   │      │   Service    │      │   + Claims  │
└─────────────┘      └──────────────┘      └─────────────┘
                            │
                            ▼
                     ┌──────────────┐
                     │ Verifiable   │
                     │ Credentials  │
                     └──────────────┘
```

## Repository Structure

```
mycelix-supplychain/
├─ rust/              # Core service + claim model + crypto
├─ ts/                # SDK, dashboard, adapters
├─ specs/             # OpenAPI + JSON schemas + examples
├─ deployments/       # Docker, K8s configs
└─ tests/             # E2E tests + test data
```

## Docs

- **OpenAPI**: [specs/openapi.yaml](specs/openapi.yaml)
- **Schemas**: [specs/schemas/](specs/schemas/)
- **Examples**: [specs/examples/](specs/examples/)
- **Contributing**: [CONTRIBUTING.md](CONTRIBUTING.md)

## Use Cases

- **Track & Trace**: End-to-end visibility from raw materials to finished goods
- **Compliance**: Auditable proof of certifications, inspections, ESG metrics
- **Anti-Counterfeiting**: Cryptographic product passports
- **Recalls**: Rapid, precise impact analysis via lineage queries

## Security

- No secrets in code
- SD-JWT/BBS+ for selective disclosure
- All claims cryptographically signed
- See [SECURITY.md](SECURITY.md) for reporting vulnerabilities

## License

Apache-2.0 - see [LICENSE](LICENSE)

## Status

**Alpha** - Reference implementation for pilots. APIs may change.

## Related Projects

- [mycelix-identity](https://github.com/Luminous-Dynamics/mycelix-identity) - DID infrastructure
- [mycelix-dkg](https://github.com/Luminous-Dynamics/mycelix-dkg) - Distributed knowledge graph
- [mycelix-consensus](https://github.com/Luminous-Dynamics/mycelix-consensus) - RB-BFT consensus

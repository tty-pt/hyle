# Publish steps

## Pre-flight

- [ ] All tests pass
  ```bash
  cargo test -p hyle -p hyle-dioxus
  cd packages/hyle-react && npm test
  cd packages/hyle-react-dom && npm test
  ```
- [ ] `examples/dioxus/Cargo.toml` has `publish = false`
- [ ] `CHANGELOG.md` date is set
- [ ] Logged in: `cargo login`, `npm login` (org: `@ttypt`)

## 1. Build npm packages

In dependency order:

```bash
cd crates/hyle/pkg-src && npm run build
cd packages/hyle-react && npm run build
cd packages/hyle-react-dom && npm run build
```

## 2. Dry runs

```bash
cargo publish --dry-run -p hyle
cargo publish --dry-run -p hyle-dioxus
cargo publish --dry-run -p hyle-dioxus-native

cd crates/hyle/pkg-src    && npm publish --dry-run --access public
cd packages/hyle-react     && npm publish --dry-run --access public
cd packages/hyle-react-dom && npm publish --dry-run --access public
```

## 3. Publish Rust crates

Wait ~30s between each for the crates.io index to propagate.

```bash
cargo publish -p hyle
cargo publish -p hyle-dioxus
cargo publish -p hyle-dioxus-native
```

## 4. Publish npm packages

```bash
cd crates/hyle/pkg-src    && npm publish --access public
cd packages/hyle-react     && npm publish --access public
cd packages/hyle-react-dom && npm publish --access public
```

## 5. Tag

```bash
git tag v0.1.0
git push origin v0.1.0
```

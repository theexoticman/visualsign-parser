# Deterministic Ordering in VisualSign

## Overview

The VisualSign library enforces **deterministic ordering** of JSON fields at compile time to ensure consistent, reproducible serialization across all platforms and implementations. While the SignablePayload is expected to be treated as an opaque, we've found that keeping the ordering deterministic helps with debugging and retaining consistency between potential `serde_json` versions. We do not advise any wallet to use this ordering as the way to display values as-is, this may change at any time in future, and only thing we guarantee that it will be deterministic. Currently this is done recursively to be alphabetical.

## Why Deterministic Ordering?

1. **Reproducibility**: The same data always produces the exact same JSON output
2. **Consistency**: All implementations across different languages produce identical output
3. **Testability**: Output can be compared byte-for-byte
4. **Security**: Prevents field ordering attacks and ensures signature consistency

## Implementation Strategy

Currently, we use **alphabetical ordering** as our deterministic strategy, but this is an implementation detail that could change in the future. The important guarantee is that ordering is deterministic and consistent.

## Compile-Time Enforcement

### The `DeterministicOrdering` Trait

All types that need deterministic JSON serialization must implement the `DeterministicOrdering` trait:

```rust
pub trait DeterministicOrdering: Serialize {
    fn verify_deterministic_ordering(&self) -> Result<(), String>;
}
```

### How It Works

1. **Custom Serialize Implementation**: Types that implement `DeterministicOrdering` must have a custom `Serialize` implementation that ensures fields are ordered deterministically.

2. **Compile-Time Checking**: Functions and methods can require the trait bound:

   ```rust
   fn process<T: DeterministicOrdering>(value: &T) { ... }
   ```

   This ensures at compile time that only types with deterministic ordering can be passed.

3. **Static Assertions**: Use const functions to verify at compile time:
   ```rust
   const _: StaticAssertDeterministic<MyType> = assert_deterministic::<MyType>();
   ```

## Current Implementations

The following types implement `DeterministicOrdering`:

- `SignablePayload`: The main payload type with alphabetically ordered fields
- `SignablePayloadField`: All field variants maintain alphabetical key ordering
- `AnnotatedPayloadField`: Flattened fields with annotations, all alphabetically ordered

## Adding New Types

When adding a new type that needs deterministic ordering:

1. **Implement Custom Serialize**:

   ```rust
   impl Serialize for MyType {
       fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
       where S: Serializer
       {
           // Use BTreeMap to ensure alphabetical ordering
           let mut map = std::collections::BTreeMap::new();
           // Add fields to map...
           // Serialize map
       }
   }
   ```

2. **Implement DeterministicOrdering**:

   ```rust
   impl DeterministicOrdering for MyType {}
   ```

3. **Add Compile-Time Test**:
   ```rust
   #[test]
   fn test_my_type_deterministic_ordering() {
       fn assert_deterministic<T: DeterministicOrdering>(_: &T) {}
       let instance = MyType::new();
       assert_deterministic(&instance);
       assert!(instance.verify_deterministic_ordering().is_ok());
   }
   ```

## What Gets Caught at Compile Time?

1. **Missing Trait Implementation**: If a type is used where `DeterministicOrdering` is required but doesn't implement it, compilation fails.

2. **Type Safety**: Functions requiring deterministic ordering won't accept types without it.

3. **API Boundaries**: Public APIs can enforce that all inputs/outputs maintain deterministic ordering.

## What Still Needs Runtime Checking?

While the trait system ensures types _claim_ to implement deterministic ordering, the actual correctness of the implementation should be verified with tests:

1. **Unit Tests**: Verify that serialized JSON actually has fields in the expected order
2. **Integration Tests**: Compare output against known-good test vectors
3. **Property Tests**: Use property-based testing to verify ordering invariants

## Future Improvements

1. **Procedural Macro**: Create a derive macro that automatically generates both `Serialize` and `DeterministicOrdering` implementations:

   ```rust
   #[derive(DeterministicSerialize)]
   struct MyType { ... }
   ```

2. **Const Verification**: Use const evaluation to verify ordering at compile time where possible.

3. **Lint Rules**: Custom lints to catch common mistakes in manual implementations.

## Examples

### Basic Usage

```rust
use visualsign::DeterministicOrdering;

// This function only accepts types with deterministic ordering
fn sign_payload<T: DeterministicOrdering>(payload: &T) -> Result<Signature, Error> {
    // Guaranteed to produce deterministic JSON
    let json = serde_json::to_string(payload)?;
    // Sign the deterministic output...
}

// Won't compile if SignablePayload doesn't implement DeterministicOrdering:
let payload = SignablePayload::new(...);
let signature = sign_payload(&payload);  // Compile-time checked!
```

### Complete Example

See [`examples/compile_time_check.rs`](../examples/compile_time_check.rs) for a complete working example demonstrating:
- How compile-time checking prevents using types without `DeterministicOrdering`
- Static assertions for compile-time verification
- What happens when you try to use a type that doesn't implement the trait

## Conclusion

The `DeterministicOrdering` trait provides compile-time guarantees that types implement deterministic field ordering, catching potential issues before runtime. Combined with proper testing, this ensures reliable, reproducible JSON serialization throughout the VisualSign system.

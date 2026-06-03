program test.aleo {
    mapping counter: u32 => u32;

    // A `Final` output lowers to an Aleo `future`, which has no visibility, so a `public`
    // (or `private`/`constant`) modifier on it must be rejected.
    fn increment(public amount: u32) -> public Final {
        return final { finalize(amount); };
    }
}

final fn finalize(amount: u32) {
    Mapping::set(counter, 0u32, amount);
}

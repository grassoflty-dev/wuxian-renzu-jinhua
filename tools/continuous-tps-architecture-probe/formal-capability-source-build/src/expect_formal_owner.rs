// Deliberate compile-fail expectation. Never a stub or formal implementation.
use wuxian_horror_ch1::capability_owner::CapabilityOwner;

fn main() {
    let _ = std::mem::size_of::<CapabilityOwner>();
}

use star_frame_idl::verifier::verify_idl_definitions;

fn main() {
    let _ = verify_idl_definitions(std::iter::empty::<&star_frame_idl::IdlDefinition>());
}

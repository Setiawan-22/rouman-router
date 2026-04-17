use rcgen::generate_simple_self_signed; fn main() { let cert = generate_simple_self_signed(vec!["a".to_string()]).unwrap(); let x: () = cert.signing_key; }

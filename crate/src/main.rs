fn main() {
    println!("Running this.env as binary...");

    // Aquí puedes inicializar tu entorno:
    let env = this_env::Env::new(None).expect("Failed to initialize Env");
    println!("Env initialized: {:?}", env.status());
}
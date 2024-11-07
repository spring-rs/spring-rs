pub const BANNER: &str = r"
            ⌡
          _@▓▄                             ___
     _,p@▒▒É▓▓L                            ▀▀▀ª
  ⌡▒▒▒▓▒▒É▒▓▓▓▓  ▄█████  ████████_  ██████ ███ª █████████,  ▄████████
 ▄▒▓▒▒▓▒▒É▓▓▓▓▓  ▓██▄_   ███~ ~███, ███▀ª~ ███─ ▓███~~▓███ ███▀~~▓██▀
δ▓▒▓▒▒▓▒▓▓▓▓▓▓█   ~▀███  ███_ _███'║███    ███  ███ª  ▓██▀ ███▄ _███L
δ▓▒▓▒▒▓▓▓▓▓▓▓▀  Γ█████▀ ╔███▀████ª δ███   ε███  ███   ▓██L ~▀███▀███ª
└▓▓▓▓▓▓▓▓▓▀ª`           Σ███                                ▄▄▄▄▄███
 ▓▀▀▀ⁿª^                ≡▀▀▀                                ▀▀▀▀▀▀▀~";

pub fn print_banner() {
    println!("{BANNER}");
    println!(" :: Spring ::                ({v3.3.1})");
    if config.logger.enable {
        println!("     logger: {}", config.logger.level.to_string().green());
    } else {
        println!("     logger: {}", "disabled".bright_red());
    }
    if cfg!(debug_assertions) {
        println!("compilation: {}", "debug".bright_red());
    } else {
        println!("compilation: {}", "release".green());
    }
}
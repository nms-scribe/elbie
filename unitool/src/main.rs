// A simple tool to manipulate strings and unicode numbers for input strings. Will be obsolete if I can ever get https://aur.archlinux.org/packages/uniutils to install.


fn main() {
    let mut args = std::env::args().skip(1);
    if let Some(command) = args.next() {
        match command.as_str() {
            "escape" => {
                for arg in args {
                    for c in arg.chars() {
                        print!("{}",c.escape_unicode());
                    }
                    println!();
                }
            },
            "encode" => {
                for arg in args {
                    if arg.starts_with('.') {
                        print!("{}",arg.trim_start_matches('.'));
                    } else if let Ok(code) = u32::from_str_radix(&arg,16) {
                        let char = char::from_u32(code).unwrap_or('\u{FFFD}');
                        print!("{char}");
                    } else {
                        eprintln!("invalid hex code: '{arg}'");
                        std::process::exit(1);
                    }
                }
                eprintln!();
            }
            command => {
                eprintln!("Unknown command '{command}'");
                std::process::exit(1);
            }
        }
    } else {
        println!("Commands:");
        println!(" escape: Escapes string into unicode forms");
        println!(" encode: Encodes hex integer arguments into a string.")
    }

}

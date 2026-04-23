//! ASCII-art logo renderer.
//!
//! The logo depicts an Iron(III) Oxide (Fe₂O₃) molecule — a nod to the
//! project name *rOSt* ("Rust", as in iron oxide).

/// Print the rOSt logo to the console using ANSI colour codes.
pub fn print_logo() {
    print!("\x1b[91m");
    println!(
        "                       # ###       #######                          .                 ..."
    );
    println!(
        "                     /  /###     /       ###                    :*@@#@@+.          +@@##@@*."
    );
    println!(
        "                    /  /  ###   /         ##   #               *#:     ;@;       .@+.     ;@+"
    );
    println!(
        "                   /  ##   ###  ##        #   ##              +*         @:      @.         #:"
    );
    println!(
        "                  /  ###    ###  ###          ##              @          :*     :*          :*"
    );
    println!(
        "    ###  /###    ##   ##     ## ## ###      ########          @          ;*     :*          ;*"
    );
    println!(
        "     ###/ #### / ##   ##     ##  ### ###   ########           +#        .@.      @;        .@."
    );
    println!(
        "      ##   ###/  ##   ##     ##    ### ###    ##               +@+.  .:*@:\x1b[90m\\\x1b[91m      ##:    .+@:\x1b[90m\\"
    );
    println!(
        "      ##         ##   ##     ##      ### /##  ##              \x1b[90m/\x1b[91m .+#@@@*;  \x1b[90m\\       /\x1b[91m:*@@@@#;\x1b[90m\\  \\"
    );
    println!(
        "      ##         ##   ##     ##        #/ /## ##             \x1b[90m/  /          \\     /          \\  \\"
    );
    println!(
        "      ##          ##  ##     ##         #/ ## ##             \x1b[90m/ /           \\     /           \\  \\\x1b[98m"
    );
    println!(
        "      ##           ## #      /           # /  ##         ...\x1b[90m/  /            \\   /             \\  \\\x1b[98m"
    );
    println!(
        "      ##            ###     /  /##        /   ##       ;@@@@@*/            .*@@@*.             \x1b[90m\\\x1b[98m;#@@#+"
    );
    println!(
        "      ###            ######/  /  ########/    ##      ;@@@@@@@*           :@@@@@@@.            *@@@@@@#"
    );
    println!(
        "       ###             ###   /     #####       ##     +@@@@@@@@           #@@@@@@@+            @@@@@@@@."
    );
    println!(
        "                             |                        :@@@@@@@+           :@@@@@@@.            *@@@@@@#"
    );
    println!(
        "                              \\)                       .*@@@#;             .*@@@*.              ;#@@#+"
    );
    print!("\x1b[0m");

    println!("");
}

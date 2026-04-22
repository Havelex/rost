/// Prints the super cool rOSt logo of the application in the terminal.
/// btw. it's an Iron (III) Oxide molecule, in case you were wondering.
/// Get it? rOSt? Rust? Iron Oxide? Yeah, I know, it's a stretch, but I thought it was clever. :)

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

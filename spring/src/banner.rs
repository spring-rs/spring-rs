pub const BANNER: &str = r"
                ~?                                                                      
             .^!JP7                                    ^!!!:                            
         .:^!7?7J55~                                   J#B#7                            
     :~77?J??7??YYY?   ^?Y55YJ^ :7?JY55YJ7:   !J??~?5! 7JJJ^ :??J7^?Y5Y?^    .!?Y55YJ?7.
   :??JJ?YJ??7?JJJJ5^ ~@@@@GG#? !@@@@BB&@@@J  G@@@@@@5 G@@@! ~@@@@@&@@@@@! .Y&@@&##@@@@^
  :YJJY??YJ???JJ?JYP^ :G@@&J^   !@@@J  .G@@@! G@@@5:.. B@@@^ !@@@&!.:B@@@? 5@@@J. .#@@&.
  ?YJYYJ?YJ??JJJY555.   ^Y&@@G: ?@@@5  .G@@@! B@@&.    #@@@^ ?@@@Y   5@@@! #@@@~  7@@@B 
  JYJYYJ?JJJYYYY555^ .BBPP@@@@^ J@@@@#B&@@@5 .&@@#.   :@@@@: 5@@@?   B@@@~ !&@@@##&@@@G 
  ?YJJYJJYY55555Y7:   7Y5P5Y7:  5@@@77Y55J~  .JJJ?    :JJJJ. 7JJJ^   ?JJJ:  :!?JJ!?@@@G 
  !5YY55P555YJ7^.               B@@@^                                       YPYJJ5&@@@J 
  ^PYJJ?7!~:.                  .PGGG:                                       ?GBB###G57  
  :^";

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
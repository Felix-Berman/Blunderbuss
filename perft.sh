#!/usr/bin/expect

set depth [lindex $argv 0]
set fen [lindex $argv 1]
set moves [join [lrange $argv 2 end]]
set init "position fen $fen moves $moves\r"
set perft "perft $depth\rquit\r"

spawn -noecho ./target/release/chess

log_user 0
send -- $init
expect $init
send -- $perft
expect $perft

log_user 1
set output $expect_out(buffer)
set idx [string first "quit" $output]
puts [string range $output $idx+6 end-1]
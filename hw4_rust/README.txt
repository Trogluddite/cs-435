Ok, so I worked through a simpler version of this in C to prep for the test,
but it seemed like a good opportunity topractice more Rust (with less 
frustration than the whole game server), and it seemed like a good chance to
gin up a simple python client. 

The Python client is dumb-simple; it's just 'one and done' (select your option
then do the thing).

I've got the Rust server running in isoptera on port 5006 (if it doesn't crash)
you can hit it with my silly python client with
`./client.py isoptera.lcsc.edu 5006`

But if it does crash, I included some client & server output to show what either
side dumps to stdout.


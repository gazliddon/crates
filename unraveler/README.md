# Combinatory parsing lib

Allows you to parse a collection of arbitary tokens

I wanted to parse to an AST in two phases, lexing to tokens and then parsing the tokens into an AST

It meant I needed something that would parse over a collection of arbitary tokens

I like Nom but found it confusing where trying to operate over text that wasn't characters

So I started writing this



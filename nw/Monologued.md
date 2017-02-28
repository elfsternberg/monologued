\documentclass{article}\usepackage{noweb}\pagestyle{noweb}\noweboptions{}\begin{document}\nwfilename{Monologued.nw}\nwbegindocs{0}% -*- Mode: noweb; noweb-code-mode: rust-mode ; noweb-doc-mode: markdown-mode -*-% ===> this file was generated automatically by noweave --- better not edit it

# Monologued is an RFC-1288 server.

Monologued is an RFC-1288 (RUIP: Remote User Information Protocol)
server with a restrict (Q1 only) syntax.  It has been mostly written as
an exercise in Rust to learn the MIO (Metal I/O) and libc interfaces, as
well as Rust in general.

RUIP offers two syntaxes

    {Q1}    ::= [{W}|{W}{S}{U}]{C}
    {Q2}    ::= [{W}{S}][{U}]{H}{C}
    {U}     ::= username
    {H}     ::= @hostname | @hostname{H}
    {W}     ::= /W
    {S}     ::= <SP> | <SP>{S}
    {C}     ::= <CRLF>

Monologued supports only the Q1 query, and can be restricted further to
limit the Q1 query only to those users in a defined group, or even to a
list of usernames specified in a file or on the command line.  It
delivers only a user's `.plan` file.  Monologued keeps a small, command
line-limited collection of plans in a cache, to further support
displaying them on request.

In the spirit of traditional IP protocol rules, Monologued is fairly
liberal in its treatment of {S} and {C}, attempting to make sense of
them.  It's input buffer is limited to 1KB by default; the maximum
output size of a .plan file is 128KB.  Changing these limits would
require a recompile.

## Literate Programming

Note: this program was written with the
[Literate Programming](http://en.wikipedia.org/wiki/Literate_programming)
toolkit [Noweb](http://www.cs.tufts.edu/~nr/noweb/). Where you see
something that looks like \<\<this\>\>, it is a placeholder for code
described elsewhere in the document.  Placeholders with an equal sign at
the end of them indicate the place where that code is defined.  The link
(U->) indicates that the code you're seeing is used later in the
document, and (<-U) indicates it was used earlier but is being defined
here.

Instructions on generating the code and documentation from this document
are provided in the accompanying Makefile.




\nwenddocs{}\end{document}


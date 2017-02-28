.SUFFIXES: .nw .rs .pdf .html .tex 

NOTANGLE=		notangle
NOWEAVE=		noweave
CARGO=			cargo

all: target/debug/Monologued

.nw.html:
	$(NOWEAVE) -filter l2h -delay -index -autodefs c -html $*.nw > $*.html

.nw.tex:
	$(NOWEAVE) -x -delay $*.nw | sed 's/\\<\\<this\\>\\>/<<this>>/' > $*.tex 			#$

.tex.pdf:
	xelatex $*.tex; \
	while grep -s 'Rerun to get cross-references right' $*.log; \
        do \
		xelatex *$.tex; \
	done

.nw.rs:

src/plan/: nw/Monologued.nw
	mkdir -p $@

src/plan/mod.rs src/plan/plan.rs: nw/Monologued.nw src/plan/
	$(NOTANGLE) -c -R$(subst src/,,$@ ) $< > $@

target/debug/Monologued: src/*.rs
	cargo build

run: target/debug/Monologued
	cargo run

clean:
	- rm -f *.tex *.dvi *.aux *.toc *.log *.out *.html *.js

realclean: clean
	- rm -f *.pdf


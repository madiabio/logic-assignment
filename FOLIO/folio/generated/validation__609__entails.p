fof(premise_1,axiom,(! [X] : (romancelanguage(X) => indoeuropeanlanguage(X)))).
fof(premise_2,axiom,(! [X] : (romancelanguage(X) => memberof(X, languagefamily)))).
fof(premise_3,axiom,(! [X] : (! [Y] : (! [Z] : ((memberof(X, Z) & memberof(Y, Z)) => (related(X, Y) & related(Y, X))))))).
fof(premise_4,axiom,(romancelanguage(french) & romancelanguage(spanish))).
fof(premise_5,axiom,related(german, spanish)).
fof(premise_6,axiom,(! [X] : (language(X) => ~(related(basque, X))))).
fof(conclusion,conjecture,romancelanguage(german)).

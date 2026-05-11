fof(premise_1,axiom,(! [X] : (book(X) => contains(X, knowledge)))).
fof(premise_2,axiom,(! [X] : (! [Y] : (readbook(X, Y) => gains(X, knowledge))))).
fof(premise_3,axiom,(! [X] : (gains(X, knowledge) => smarter(X)))).
fof(premise_4,axiom,(readbook(harry, walden) & book(walden))).
fof(conclusion,conjecture,smarter(harry)).

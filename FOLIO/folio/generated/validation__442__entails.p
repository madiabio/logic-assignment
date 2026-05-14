fof(premise_1,axiom,flyto(susan, lgaairport)).
fof(premise_2,axiom,(! [X] : (! [Y] : ((flyfrom(X, Y) | flyto(X, Y)) & ~((flyfrom(X, Y) & flyto(X, Y))))))).
fof(premise_3,axiom,flyfrom(john, lgaairport)).
fof(conclusion,conjecture,flyfrom(susan, lgaairport)).

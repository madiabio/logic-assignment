fof(premise_1,axiom,(! [X] : (square(X) => foursided(X)))).
fof(premise_2,axiom,(! [X] : (foursided(X) => shape(X)))).
fof(conclusion_negated,conjecture,~((! [X] : (square(X) => shape(X))))).

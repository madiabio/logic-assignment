fof(premise_1,axiom,(! [X] : (cat(X) => mammal(X)))).
fof(premise_2,axiom,(? [X] : (pet(X) & ~(mammal(X))))).
fof(conclusion_negated,conjecture,~((! [X] : (pet(X) => ~(cat(X)))))).

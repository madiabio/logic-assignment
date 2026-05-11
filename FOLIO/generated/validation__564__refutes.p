fof(premise_1,axiom,(? [X] : (occurin(monkeypoxvirus, X) & get(X, monkeypoxvirus)))).
fof(premise_2,axiom,(? [X] : (animal(X) & occurin(monkeypoxvirus, X)))).
fof(premise_3,axiom,(! [X] : (human(X) => mammal(X)))).
fof(premise_4,axiom,(! [X] : (mammal(X) => animal(X)))).
fof(premise_5,axiom,(? [X] : (symptonof(X, monkeypoxvirus) & (((fever(X) | headache(X)) | musclepain(X)) | tired(X))))).
fof(premise_6,axiom,(! [X] : ((human(X) & get(X, flu)) => feel(X, tired)))).
fof(conclusion_negated,conjecture,~((! [X] : (human(X) => ~(get(X, flu)))))).

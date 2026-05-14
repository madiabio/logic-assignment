fof(premise_1,axiom,(! [X] : (niceto(X, animal) => ~(meanto(X, animal))))).
fof(premise_2,axiom,(? [X] : (grumpy(X) & meanto(X, animal)))).
fof(premise_3,axiom,(! [X] : (animallover(X) => niceto(X, animal)))).
fof(premise_4,axiom,(! [X] : (petowner(X) => animallover(X)))).
fof(premise_5,axiom,petowner(tom)).
fof(conclusion,conjecture,grumpy(tom)).

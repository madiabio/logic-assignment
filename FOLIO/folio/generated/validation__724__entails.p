fof(premise_1,axiom,(! [X] : (event(X) => ((happy(X) | sad(X)) & ~((happy(X) & sad(X))))))).
fof(premise_2,axiom,(? [X] : (event(X) & happy(X)))).
fof(conclusion,conjecture,(! [X] : (event(X) => sad(X)))).

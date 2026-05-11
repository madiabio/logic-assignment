fof(premise_1,axiom,(! [X] : (vehicleregistrationplatein(X, istanbul) => beginwith(X, num34)))).
fof(premise_2,axiom,(! [X] : (~(beginwith(X, num34)) => ~(fromistanbul(X))))).
fof(premise_3,axiom,(? [X] : (owns(joe, X) & vehicleregistrationplatein(X, istanbul)))).
fof(premise_4,axiom,(? [X] : (owns(tom, X) & beginwith(X, num35)))).
fof(premise_5,axiom,(! [X] : (beginwith(X, num35) => ~(beginwith(X, num34))))).
fof(conclusion_negated,conjecture,~((? [X] : (owns(tom, X) & vehicleregistrationplatein(X, istanbul))))).

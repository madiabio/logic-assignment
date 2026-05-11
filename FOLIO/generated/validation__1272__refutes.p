fof(premise_1,axiom,(! [X] : (brownswisscattle(X) => cow(X)))).
fof(premise_2,axiom,(? [X] : (pet(X) & brownswisscattle(X)))).
fof(premise_3,axiom,(! [X] : (cow(X) => domesticatedanimal(X)))).
fof(premise_4,axiom,(! [X] : (aligator(X) => ~(domesticatedanimal(X))))).
fof(premise_5,axiom,aligator(ted)).
fof(conclusion_negated,conjecture,~((pet(ted) & brownswisscattle(ted)))).

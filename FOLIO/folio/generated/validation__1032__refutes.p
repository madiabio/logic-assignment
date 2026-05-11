fof(premise_1,axiom,(! [X] : (deadlydiseases(X) => comewith(X, lowsurvivalrate)))).
fof(premise_2,axiom,(! [X] : (severecancer(X) => deadlydiseases(X)))).
fof(premise_3,axiom,(! [X] : (bileductcancer(X) => severecancer(X)))).
fof(premise_4,axiom,(! [X] : (cholangiocarcinoma(X) => bileductcancer(X)))).
fof(premise_5,axiom,(! [X] : (mildflu(X) => ~(comewith(X, lowsurvivalrate))))).
fof(premise_6,axiom,~((bileductcancer(colorectalcancer) & comewith(colorectalcancer, lowsurvivalrate)))).
fof(conclusion_negated,conjecture,~(severecancer(colorectalcancer))).

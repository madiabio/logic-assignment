fof(premise_1,axiom,(! [X] : (animal(X) => ((invertebrate(X) | vertebrate(X)) & ~((invertebrate(X) & vertebrate(X))))))).
fof(premise_2,axiom,(! [X] : ((animal(X) & with(X, backbone)) => reproduceby(X, male_and_femalemating)))).
fof(premise_3,axiom,(! [X] : ((animal(X) & vertebrate(X)) => with(X, backbone)))).
fof(premise_4,axiom,(! [X] : (bee(X) => ~(reproduceby(X, male_and_femalemating))))).
fof(premise_5,axiom,(! [X] : (queenbee(X) => bee(X)))).
fof(premise_6,axiom,bee(harry)).
fof(conclusion_negated,conjecture,~((~(((invertebrate(harry) | with(harry, backbone)) & ~((invertebrate(harry) & with(harry, backbone))))) => (~(invertebrate(harry)) & ~(queenbee(harry)))))).

fof(premise_1,axiom,(mountainrange(thepicurismountains) & ((in(thepicurismountains, newmexico) | in(thepicurismountains, texas)) & ~((in(thepicurismountains, newmexico) & in(thepicurismountains, texas)))))).
fof(premise_2,axiom,visited(juandeonate, thepicurismountains)).
fof(premise_3,axiom,((in(hardingpegmatitemine, thepicurismountains) & mine(hardingpegmatitemine)) & donated(hardingpegmatitemine))).
fof(premise_4,axiom,(! [X] : (! [Y] : ((((mine(X) & donated(X)) & in(X, Y)) & mountainrange(Y)) => ~(in(Y, texas)))))).
fof(conclusion,conjecture,(? [X] : ((visited(juandeonate, X) & mountainrange(X)) & in(X, newmexico)))).

fof(premise_1,axiom,(! [X] : (? [Y] : (listedin(X, yelprecommendation) => (negativereview(Y) & ~(receive(X, Y))))))).
fof(premise_2,axiom,(! [X] : (! [Y] : ((haverating(X, Y) & greaterthan(Y, c_4)) => listedin(X, yelprecommendation))))).
fof(premise_3,axiom,(? [X] : (? [Y] : (~(provide(X, takeoutservice)) => (negativereview(Y) & receive(X, Y)))))).
fof(premise_4,axiom,(! [X] : (! [Y] : (popularamong(X, localresidents) => (haverating(X, Y) & greaterthan(Y, c_4)))))).
fof(premise_5,axiom,(! [X] : (((haverating(hamdenplazasubway, X) & greaterthan(X, c_4)) | popularamong(hamdenplazasubway, localresidents)) & ~(((haverating(hamdenplazasubway, X) & greaterthan(X, c_4)) & popularamong(hamdenplazasubway, localresidents)))))).
fof(conclusion,conjecture,~(provide(hamdenplazasubway, takeoutservice))).

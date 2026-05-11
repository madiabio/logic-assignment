fof(premise_1,axiom,(! [X] : (! [Y] : (((superheromovie(X) & in(Y, X)) & goodguy(Y)) => wins(Y))))).
fof(premise_2,axiom,superheromovie(thesurprisingadventuresofsirdigbychickencaesar)).
fof(premise_3,axiom,(! [X] : (! [Y] : ((goodguy(X) & fight(X, Y)) <=> (badguy(Y) & fight(Y, X)))))).
fof(premise_4,axiom,fight(sirdigby, sirdigbysnemesis)).
fof(premise_5,axiom,(! [X] : (! [Y] : ((superheromovie(X) & namedafter(X, Y)) => goodguy(Y))))).
fof(premise_6,axiom,namedafter(thesurprisingadventuresofsirdigbychickencaesar, sirdigby)).
fof(premise_7,axiom,(! [X] : (! [Y] : ((fights(X, Y) & win(X)) => ~(wins(Y)))))).
fof(premise_8,axiom,(! [X] : (! [Y] : ((superheromovie(X) & namedafter(X, Y)) => in(Y, X))))).
fof(conclusion,conjecture,~(win(sirdigbysnemesis))).

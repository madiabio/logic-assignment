fof(premise_1,axiom,(? [X] : ((professional(X) & basketballplayer(X)) & ~(americannational(X))))).
fof(premise_2,axiom,(! [X] : ((professional(X) & basketballplayer(X)) => cando(X, jumpshot)))).
fof(premise_3,axiom,(! [X] : (cando(X, jumpshot) => leapstraightintoair(X)))).
fof(premise_4,axiom,(! [X] : (leapstraightintoair(X) => activate(X, legmuscle)))).
fof(premise_5,axiom,~(activate(yuri, legmuscle))).
fof(conclusion,conjecture,~(((americannational(yuri) & professional(yuri)) & basketballplayer(yuri)))).

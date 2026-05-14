fof(premise_1,axiom,(! [X] : (rankedhighlyby(X, womenstennisassociation) => mostactiveplayerin(X, majortennis)))).
fof(premise_2,axiom,(! [X] : ((lostto(X, wi_tek) & at(X, rolandgarros2022)) => rankedhighlyby(X, womenstennisassociation)))).
fof(premise_3,axiom,(! [X] : ((femaletennisplayer(X) & at(X, rolandgarros2022)) => (lostto(X, wi_tek) & at(X, rolandgarros2022))))).
fof(premise_4,axiom,(! [X] : ((tennisplayer(X) & at(X, rolandgarros2022)) => (((female(X) & tennisplayer(X)) | (male(X) & tennisplayer(X))) & ~(((female(X) & tennisplayer(X)) & (male(X) & tennisplayer(X)))))))).
fof(premise_5,axiom,(! [X] : (((male(X) & tennisplayer(X)) & at(X, rolandgarros2022)) => (lostto(X, wi_tek) & at(X, rolandgarros2022))))).
fof(premise_6,axiom,((rankedhighlyby(cocogauff, womenstennisassociation) | (lostto(cocogauff, wi_tek) & lostat(cocogauff, rolandgarros2022))) => ~(((male(cocogauff) & tennisplayer(cocogauff)) & atrolandgarros2022(cocogauff))))).
fof(premise_7,axiom,(tennisplayer(cocogauff) & at(cocogauff, rolandgarros2022))).
fof(conclusion_negated,conjecture,~((~((lostto(cocogauff, wi_tek) & at(cocogauff, rolandgarros2022))) | ~(mostactiveplayerin(cocogauff, majortennis))))).

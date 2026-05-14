fof(premise_1,axiom,((cove(barutincove) & namedafter(barutincove, barutinsettlement)) & locatedin(barutinsettlement, bulgaria))).
fof(premise_2,axiom,locatedin(barutincove, snowisland)).
fof(premise_3,axiom,((locatedin(snowisland, southshetlandislands) & locatedin(greenwichisland, southshetlandislands)) & locatedin(deceptionisland, southshetlandislands))).
fof(premise_4,axiom,locatedin(southshetlandislands, antarctica)).
fof(premise_5,axiom,(! [X] : (! [Y] : (! [Z] : ((locatedin(X, Y) & locatedin(Y, Z)) => locatedin(X, Z)))))).
fof(conclusion,conjecture,(? [X] : (? [Y] : (((cove(X) & locatedin(X, antarctica)) & nameafter(X, Y)) & locatedin(Y, bulgaria))))).

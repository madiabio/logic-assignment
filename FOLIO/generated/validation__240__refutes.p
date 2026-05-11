fof(premise_1,axiom,striker(robertlewandowski)).
fof(premise_2,axiom,(! [X] : (striker(X) => soccerplayer(X)))).
fof(premise_3,axiom,left(robertlewandowski, bayernmunchen)).
fof(premise_4,axiom,(! [X] : (! [Y] : (left(X, Y) => ~(playsfor(X, Y)))))).
fof(conclusion_negated,conjecture,~(soccerplayer(robertlewandowski))).

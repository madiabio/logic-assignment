fof(premise_1,axiom,capitalof(beijing, peoplesrepublicofchina)).
fof(premise_2,axiom,(? [X] : (capitalof(beijing, X) => worldsmostpopulousnation(X)))).
fof(premise_3,axiom,locatedin(beijing, northernchina)).
fof(premise_4,axiom,(hosted(beijing, c_2008summerolympics) & hosted(beijing, c_2008summerparalympicgames))).
fof(premise_5,axiom,(((hosted(beijing, summerolympics) & hosted(beijing, winterolympics)) & hosted(beijing, summerparalympicgames)) & hosted(beijing, winterparalympicgames))).
fof(premise_6,axiom,(? [X] : ((university(X) & inbeijing(X)) & consistentlyrankamongthebestin(X, theworld)))).
fof(conclusion,conjecture,locatedin(beijing, southernchina)).

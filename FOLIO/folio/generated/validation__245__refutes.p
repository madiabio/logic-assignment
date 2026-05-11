fof(premise_1,axiom,(publishinghouse(newvesselpress) & specializesintranslatingintoenglish(newvesselpress, foreignliterature))).
fof(premise_2,axiom,(! [X] : ((book(X) & publishedby(X, newvesselpress)) => in(X, english)))).
fof(premise_3,axiom,(book(neapolitanchronicles) & publishedby(neapolitanchronicles, newvesselpress))).
fof(premise_4,axiom,translatedfrom(neapolitanchronicles, italian)).
fof(premise_5,axiom,(book(palaceofflies) & publishedby(palaceofflies, newvesselpress))).
fof(conclusion_negated,conjecture,~(translatedfrom(palaceofflies, italian))).

fof(premise_1,axiom,(! [X] : (digitalmedia(X) => ~(analogmedia(X))))).
fof(premise_2,axiom,(! [X] : (printedtext(X) => analogmedia(X)))).
fof(premise_3,axiom,(! [X] : (streamingservice(X) => digitalmedia(X)))).
fof(premise_4,axiom,(! [X] : (hardcoverbook(X) => printedtext(X)))).
fof(premise_5,axiom,(streamingservice(c_1984) => hardcoverbook(c_1984))).
fof(conclusion_negated,conjecture,~(~(streamingservice(y1984)))).

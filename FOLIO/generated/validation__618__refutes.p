fof(premise_1,axiom,(likemusic(george) => wanttocompose(george))).
fof(premise_2,axiom,(accesstoprogram(george) => cancompose(george))).
fof(premise_3,axiom,((wanttocompose(george) & cancompose(george)) => willcompose(george))).
fof(conclusion_negated,conjecture,~(((likemusic(george) & accesstoprogram(george)) => willcompose(george)))).

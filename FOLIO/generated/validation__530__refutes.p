fof(premise_1,axiom,(netflixshow(strangerthings) & popular(strangerthings))).
fof(premise_2,axiom,(! [X] : ((netflixshow(X) & popular(X)) => bingewatch(karen, X)))).
fof(premise_3,axiom,(! [X] : ((netflixshow(X) & bingewatch(karen, X)) <=> download(karen, X)))).
fof(premise_4,axiom,~(download(karen, blackmirror))).
fof(premise_5,axiom,netflixshow(blackmirror)).
fof(premise_6,axiom,(! [X] : ((netflixshow(X) & bingewatch(karen, X)) => sharewith(karen, X, lisa)))).
fof(conclusion_negated,conjecture,~(sharewith(karen, strangerthings, lisa))).

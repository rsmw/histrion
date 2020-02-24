# Histrion

Histrion is a simulation engine for historical timelines on an interstellar scale. By representing events as data, it aspires to provide writers with the same capacity to analyze their stories as programmers get today with their code.

Being at such an early stage of development, Histrion supports only a few core features, and comes with almost no documentation. The current interface is entirely programmatic (you must write code to make it work). At some point in the future, it will support a text-based DSL to make it easier to input events, which might look like the following:

```histrion-saga
spawn Mars

foo = 2

as Mars do
    wait 1hr
    trace foo
    transmit #arrived(Mars)
done

listen #arrived(Mars)

halt
```

Someday it may also support an entirely graphical interface for creating and editing events. There's no concrete roadmap, so don't hold your breath.


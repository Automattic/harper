#set page(
  paper: "us-letter",
  columns: 2,
)

#let titleblock(
  title: "Default Title",
  authors: ("Author 1", "Author 2"),
  abstract: [*This is content*],
) = {
  set page(
    header: context {
      if counter(page).get().first() > 1 [
        #counter(page).get().first() of #counter(page).final().at(0)
        #h(1fr)
        #title
      ]
    },
  )
  place(
    top + center,
    float: true,
    scope: "parent",
    clearance: 2em,
  )[
    #align(center, text(17pt)[
      *#title*
    ])

    #let authors = authors.filter(x => x.len() > 0)
    #let count = authors.len()
    #let authors_slice = authors.slice(0, calc.min(count, 3))
    _#if count > 3 {
        // et al. isn't parsed properly, but this isn't the fault of the typst
        // parser
        // authors_slice.push("et al.")
        authors_slice.join(", ")
      } else {
        authors_slice.join(", ", last: ", and ")
      }
    _

    #par(justify: false)[
      *Abstract* \
      #abstract
    ]
  ]
}

#show: doc => [
  #titleblock(
    title: "A fluid dynamic model for glacier flow",
    authors: ("Grant Lemons", "John Doe", "Jane Doe"),
    abstract: lorem(80)
  )
  #doc
]

= Introduction
#lorem(300)

= Related Work
#lorem(200)

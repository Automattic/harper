#let template(
  title: "Default Title",
  authors: ("Author 1", "Author 2"),
  abstract: [*This is content*],
  body,
) = {
  set document(date: none)
  set par(justify: true)
  set page(
    paper: "us-letter",
    columns: 2,
    number-align: top,
    numbering: (..n) => if n.pos().first() > 1 {
      n.pos().map(str).join(" of ") + h(1fr) + title
    },
  )

  place(
    top + center,
    float: true,
    scope: "parent",
    clearance: 2em,
  )[
    #text(17pt, strong(title))

    #let authors-line = if authors.len() > 3 {
      // "et al." isn't parsed properly, but this isn't the fault of the Typst
      // parser.
      // authors-max3.push("et al.")
      authors => authors.join(", ")
    } else {
      authors => authors.join(", ", last: ", and ")
    }
    #emph(authors-line(authors.slice(0, calc.min(authors.len(), 3))))

    #par(justify: false)[
      *Abstract* \
      #abstract
    ]
  ]

  body
}

#show: template.with(
  title: "A fluid dynamic model for glacier flow",
  authors: ("Grant Lemons", "John Doe", "Jane Doe"),
  abstract: lorem(80),
)

= Introduction
#lorem(300)

= Related Work
#lorem(200)

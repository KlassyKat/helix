// From insert_newline we ask the question are we between a pair?
// This file answers with the ranges of the closest surrounding pair
// In insert_newline we then ask Is the end of the start range 1 less than the start of the end range
// If yes, we do the bracket pair behavior of opening an indented line and putting the closing pair on the next line matching the indent of the open pair

// Query doc pairs

// Find closest surrounding pair -> (Range, Range)

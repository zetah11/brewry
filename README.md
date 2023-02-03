# Brewry

Object orientation for the sleepy developer.

    variant NamePart is Stringable

        variant Identifier
            var name String

            class Value end
            class Type end
        end

        variant Invalid
    
        function to_string(this &) String
            case this
                is Invalid
                    return "<invalid>"
                end

                is Identifier.Value(name)
                    return "var " + name
                end

                is Identifier.Type(name)
                    return "type " + name
                end
            end
        end

    end

Brewry is statically typed and class-based (no message passing, sorry!). Unlike
most other high-level, Brewry is pass-by-value by default. References must be
explicitly annotated (that's the `this &` above). This makes it a lot easier to
reason about where state goes and who can mutate what.

_st() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="st"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        st)
            opts="-V -m -d -a -z -h --cheet --completions --man --mcp --mcp-tools --mcp-config --version --terminal --rename-project --input --mode --find --type --entry-type --min-size --max-size --newer-than --older-than --depth --no-ignore --no-default-ignore --all --show-ignored --everything --show-filesystems --no-emoji --compress --mcp-optimize --compact --path-mode --color --ai-json --stream --sse-server --sse-port --search --semantic --mermaid-style --no-markdown-mermaid --no-markdown-tables --no-markdown-pie-charts --focus --relations-filter --sort --top --show-private --view-diffs --cleanup-diffs --help [PATH]"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --completions)
                    COMPREPLY=($(compgen -W "bash elvish fish powershell zsh" -- "${cur}"))
                    return 0
                    ;;
                --rename-project)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --input)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --mode)
                    COMPREPLY=($(compgen -W "auto classic hex json ls ai stats csv tsv digest quantum semantic mermaid markdown summary summary-ai relations quantum-semantic waste marqant sse function-markdown" -- "${cur}"))
                    return 0
                    ;;
                -m)
                    COMPREPLY=($(compgen -W "auto classic hex json ls ai stats csv tsv digest quantum semantic mermaid markdown summary summary-ai relations quantum-semantic waste marqant sse function-markdown" -- "${cur}"))
                    return 0
                    ;;
                --find)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --type)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --entry-type)
                    COMPREPLY=($(compgen -W "f d" -- "${cur}"))
                    return 0
                    ;;
                --min-size)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-size)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --newer-than)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --older-than)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --depth)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --path-mode)
                    COMPREPLY=($(compgen -W "off relative full" -- "${cur}"))
                    return 0
                    ;;
                --color)
                    COMPREPLY=($(compgen -W "always never auto" -- "${cur}"))
                    return 0
                    ;;
                --sse-port)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --search)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --mermaid-style)
                    COMPREPLY=($(compgen -W "flowchart mindmap gitgraph treemap" -- "${cur}"))
                    return 0
                    ;;
                --focus)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --relations-filter)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --sort)
                    COMPREPLY=($(compgen -W "a-to-z z-to-a largest smallest newest oldest type name size date" -- "${cur}"))
                    return 0
                    ;;
                --top)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cleanup-diffs)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _st -o nosort -o bashdefault -o default st
else
    complete -F _st -o bashdefault -o default st
fi

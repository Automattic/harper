import { IconName, ItemView, WorkspaceLeaf, Menu, setIcon } from 'obsidian';
import HarperPlugin from './index';
import { LINT_KIND_COLORS } from './lintKindColor';

export const Harper_Sidebar_View = "harper-sidebar-view";

export class SidebarView extends ItemView {
    constructor(leaf : WorkspaceLeaf, private plugin: HarperPlugin){
        super(leaf);
    }

    getViewType(){
        return Harper_Sidebar_View;
    }

    getDisplayText(){
        return "Harper Grammar";
    }

    getIcon(): IconName {
        return "book-open-check";
    }

    async onOpen(){
        this.update();
    }

    update(){
        const container = this.containerEl;
        container.empty();

        container.style.padding = "0";
        container.style.display= "flex";
        container.style.flexDirection = "column";
        container.style.justifyContent= "flex-start";

        const listContainer = document.createElement("div");

        // initial card
        const initialCard = document.createElement("div");
        initialCard.style.display= "flex";
        initialCard.style.flexDirection = "column";
        initialCard.style.padding = "12px 16px";
        initialCard.style.borderBottom = "1px solid var(--background-modifier-border)"; // add var
        initialCard.style.gap = "8px";
        
        const initialTitle = document.createElement("span");
        initialTitle.style.fontWeight = "bold";
        initialTitle.textContent =  "Loading errors...";

        initialCard.appendChild(initialTitle);
                    
        listContainer.appendChild(initialCard);

        listContainer.id = "harper-error-list";
        listContainer.style.height = "100%";
        listContainer.style.overflowY = "auto";

        container.appendChild(listContainer);

        this.registerEvent(
            this.app.workspace.on('harper:lint-updated', (errors: any[], editorView: any)=>{

                listContainer.innerHTML = "";

                if (!errors || errors.length === 0){
                    const card = document.createElement("div");
                    card.style.display= "flex";
                    card.style.flexDirection = "column";
                    card.style.padding = "12px 16px";
                    card.style.borderBottom = "1px solid var(--background-modifier-border)"; // add var
                    card.style.gap = "8px";

                    const title = document.createElement("span");
                    title.style.fontWeight = "bold";
                    title.textContent =  "No grammer errors found!";

                    card.appendChild(title);
                    
                    listContainer.appendChild(card);
                    return;
                }

                errors.forEach(error => {
                    try {
                    // find card color
                    let severityColor = "";
                    
                    if (error.markClass ){
                        const classes = error.markClass.split(' ');

                        const harperClass = classes.find((c: string) => c.startsWith('harper-lintRange-'));
                        if (harperClass){
                            const lintKind = harperClass.replace('harper-lintRange-', '');
                            if(LINT_KIND_COLORS && LINT_KIND_COLORS[lintKind]){
                                severityColor = LINT_KIND_COLORS[lintKind];
                            }
                        }
                    }

                    const card = document.createElement("div");
                    card.style.display= "flex";
                    card.style.flexDirection = "column";
                    card.style.padding = "12px 16px";
                    card.style.borderBottom = "1px solid var(--background-modifier-border)"; // add var
                    card.style.gap = "8px";
                    
                    const titleDiv = document.createElement("div");
                    titleDiv.style.display = "flex";
                    titleDiv.style.flexWrap = "wrap";
                    titleDiv.style.gap = "6px";
                    titleDiv.style.marginTop = "4px";

                    const title = document.createElement("span");
                    title.style.fontWeight = "bold";
                    title.style.color = severityColor;
                    title.textContent = error.title || "Harper Sugestion";
                    
                    const doc = editorView.state.doc;
                    const problemText = doc.sliceString(error.from, error.to);

                    // get 18 char before and after the word
                    const rawPrefix = doc.sliceString(Math.max(0, error.from-18), error.from);
                    const rawSuffix = doc.sliceString(error.to, Math.min(doc.length, error.to+18));
                    // trim to be only 3 words before and after. 
                    let prefix = rawPrefix.split(/[.!?\n]/).pop() || "";
                    let prefixWords = prefix.trim().split(/\s+/).filter(w => w.length>0);
                    prefix = prefixWords.slice(-3).join(" ");
                    if (prefix.length > 0) prefix += " ";

                    let suffix = rawSuffix.split(/[.!?\n]/)[0] || "";
                    let suffixWords = suffix.trim().split(/\s+/).filter(w => w.length>0);
                    suffix = suffixWords.slice(0,3).join(" ");
                    if (suffix.length > 0) suffix = " " + suffix;
                    
                    // text container
                    const textContainer = document.createElement("span");
                    textContainer.style.fontSize = "var(--font-ui-small)";

                    if (prefix) {
                        textContainer.appendChild(document.createTextNode(prefix));
                    }
                    
                    const boldWord = document.createElement("strong");
                    boldWord.textContent = problemText;
                    boldWord.style.color = severityColor;
                    textContainer.appendChild(boldWord);

                    if (suffix) {
                        textContainer.appendChild(document.createTextNode(suffix));
                    }

                    if (error.actions && error.actions.length > 0){
                        const actionConst = document.createElement("div");
                        actionConst.style.display = "flex";
                        actionConst.style.flexWrap = "wrap";
                        actionConst.style.gap = "6px";
                        actionConst.style.marginTop = "4px";

                        error.actions.forEach((action: any) => {
                            const btn = document.createElement("button");
                            btn.textContent = action.name;
                            btn.title = action.title;

                            btn.style.fontSize = "var(--font-ui-smaller)";
                            btn.style.cursor = "var(--cursor)";

                            btn.onclick = () => {
                                action.apply(editorView, error.from, error.to);
                            };

                            actionConst.appendChild(btn);
                        });
                        // add the options ignore diagnostic and Disable Rule in a dropdown btn
                        if (error.ignore || error.disable) {
                            const btn = document.createElement("div");
                            setIcon(btn, "more-vertical");
                            btn.style.cursor = "var(--cursor)";
                            btn.style.color = "var(--text-muted)";
                            btn.style.display = "flex";
                            btn.style.alignItems = "center";
                            btn.style.padding = "2px 4px";
                            btn.style.borderRadius = "var(--radius-s)";
                            btn.style.marginLeft = "auto";
                        
                            btn.onmouseover = () => btn.style.backgroundColor = "var(--background-modifier-hover)";
                            btn.onmouseout = () => btn.style.backgroundColor = "transparent";
                        
                            btn.onclick = (e) => {
                                const menu = new Menu();
                            
                                if (error.ignore){
                                    menu.addItem((item) => {
                                        item.setTitle("Ignore Diagnostic").setIcon("eye-off").onClick(() => {error.ignore();});
                                    });
                                }
                                if (error.disable){
                                    menu.addItem((item) => {
                                        item.setTitle("Disable Rule").setIcon("ban").onClick(() => {error.disable();});
                                    });
                                }
                            
                                menu.showAtMouseEvent(e);
                            }

                            titleDiv.appendChild(title);
                            titleDiv.appendChild(btn);
                            card.appendChild(titleDiv);
                            card.appendChild(textContainer);
                            card.appendChild(actionConst);
                        }
                        
                    }
                    listContainer.appendChild(card);
                    
                }
                catch (err){
                    console.error("Harper Sidebar failed to read: ", err, error);
                }
                });
            })
        )
    }

    async onClose(){

    }
}
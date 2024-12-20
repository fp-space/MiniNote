use crate::components::icon::{FileIcon, FolderIcon};
use crate::context::app_context::AppStateContext;
use crate::model::file_tree::FileNode;
use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(PartialEq)]
enum Tab {
    File,
    Outline
}

#[function_component(Sidebar)]
pub fn sidebar() -> Html {
    let active_tab = use_state(|| Tab::File);

    // 切换到文件视图
    let show_file = {
        let active_tab = active_tab.clone();
        Callback::from(move |_: MouseEvent| {
            active_tab.set(Tab::File);
        })
    };

    // 切换到大纲视图
    let show_outline = {
        let active_tab = active_tab.clone();
        Callback::from(move |_: MouseEvent| {
            active_tab.set(Tab::Outline);
        })
    };

    html! {
            <div class="sidebar">
                <div class="sidebar-tabs">
                    <button
                        class={if *active_tab == Tab::File { "sidebar-tab active" } else { "sidebar-tab" }}
                        onclick={show_file}
                    >
                        { "文件" }
                    </button>
                    <button
                        class={if *active_tab == Tab::Outline { "sidebar-tab active" } else { "sidebar-tab" }}
                        onclick={show_outline}
                    >
                        { "大纲" }
                    </button>
                </div>

                    <div class="sidebar-view">
                        <div class={if *active_tab == Tab::File { "tab-content active" } else { "tab-content" }}>
                            <FileView />
                        </div>
                        <div class={if *active_tab == Tab::Outline { "tab-content active" } else { "tab-content" }}>
                            <OutlineView />
                        </div>
                    </div>
            </div>
    }
}

// 大纲组件
#[function_component(OutlineView)]
fn outline_view() -> Html {
    let file_tree = FileNode::load_tree_data();

    // 初始化文件夹的展开状态
    let expanded_state = use_state(|| HashMap::<String, bool>::new());

    // html! {
    //     <div class="sidebar-content">
    //         <div class="outline-tree">
    //             {file_tree.iter().map(|node| render_file_node(node, expanded_state.clone())).collect::<Html>()}
    //         </div>
    //     </div>
    // }

    html! {
        <div class="sidebar-content">
            <div class="outline-tree">
            </div>
        </div>
    }
}


#[function_component(FileView)]
fn file_view() -> Html {
    // 存储文件树数据
    let file_tree = use_state(|| None);
    // 存储展开状态
    let expanded_state = use_state(|| HashMap::<String, bool>::new());
    let app_state_ctx = use_context::<AppStateContext>().unwrap();

    // 使用 use_effect 来加载数据，仅在组件挂载时触发一次，且只有当 file_tree 为空时加载
    {
        let file_tree = file_tree.clone();
        let expanded_state = expanded_state.clone();
        use_effect(move || {
            // 只有当 file_tree 为空时才触发请求
            if file_tree.is_none() {
                spawn_local(async move {
                    // 假设 load_tree_data 是你提供的异步加载数据函数
                    let tree = FileNode::load_tree_data().await;
                    file_tree.set(Some(tree)); // 更新文件树数据
                });
            }
            // 返回一个清理函数（组件卸载时触发）
            || {}
        });
    }

    // 渲染文件树
    let render_tree = match &*file_tree {
        Some(tree) => tree.iter().map(|node| render_file_node(node, &app_state_ctx, expanded_state.clone())).collect::<Html>(),
        None => html! { <div>{"Loading..."}</div> }, // 加载时显示 Loading
    };

    html! {
        <div class="sidebar-content">
            <div class="outline-tree">
                { render_tree }
            </div>
        </div>
    }
}


fn render_file_node(file_node: &FileNode, app_state_ctx: &AppStateContext, expanded_state: UseStateHandle<HashMap<String, bool>>) -> Html {
    let file_name = file_node.name.clone();

    // 获取当前文件夹的展开状态
    let is_expanded = expanded_state
        .get(&file_name)
        .cloned()
        .unwrap_or(false);

    // 点击事件：更新当前文件夹的展开状态
    let toggle = {
        let expanded_state = expanded_state.clone();
        let file_name = file_name.clone();
        let file_node = file_node.clone();
        let app_state_ctx = app_state_ctx.clone(); // 克隆 Rc

        Callback::from(move |_| {
            let mut state = (*expanded_state).clone();
            let is_expanded = state.entry(file_name.clone()).or_insert(false);
            *is_expanded = !*is_expanded;
            expanded_state.set(state);

            if !file_node.is_dir {
                // 仅在点击文件时更新上下文中的 selected_file
                app_state_ctx.dispatch(Some(file_node.clone()));
            }
        })
    };

    html! {
        <div class="file-node">
            // 节点头部（文件夹或文件）
            <div class="file-node-header" onclick={toggle}>
                {
                    if file_node.is_dir {
                        html! { <FolderIcon used={is_expanded} /> }
                    } else {
                        html! { <FileIcon /> }
                    }
                }
                <span class="file-node-name">{ &file_node.name }</span>
            </div>

            // 子节点（仅在文件夹展开时渲染）
            {
                if is_expanded && file_node.is_dir {
                html! {
                        <div class="file-node-children">
                            {
                                file_node.children.as_ref()
                                    .unwrap_or(&vec![])
                                    .iter()
                                    .map(|child| render_file_node(child, app_state_ctx, expanded_state.clone()))
                                    .collect::<Html>()
                            }
                        </div>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}

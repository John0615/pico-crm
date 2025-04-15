use leptos::prelude::*;


#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center h-screen bg-gray-100">
            <h1 class="text-9xl font-bold text-gray-800">404</h1>
            <p class="text-2xl text-gray-600 mt-4">页面未找到</p>
            <a href="/" class="text-blue-500 hover:underline mt-8">返回主页</a>
        </div>
    }
}

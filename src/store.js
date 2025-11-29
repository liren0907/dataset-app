import { writable } from "svelte/store";
import { browser } from '$app/environment';

// ========== Dark Mode Store ==========
// 從 localStorage 讀取，預設為 false（淺色模式）
const storedDarkMode = browser ? localStorage.getItem('darkMode') === 'true' : false;
export const darkMode = writable(storedDarkMode);

// 初始化時套用 dark class
if (browser && storedDarkMode) {
    document.documentElement.classList.add('dark');
}

export const toggleDarkMode = () => {
    darkMode.update(val => {
        const newVal = !val;
        if (browser) {
            localStorage.setItem('darkMode', String(newVal));
            if (newVal) {
                document.documentElement.classList.add('dark');
            } else {
                document.documentElement.classList.remove('dark');
            }
        }
        return newVal;
    });
};

// ========== Sidebar Store ==========
const storedSidebarOpen = browser ? localStorage.getItem('sidebarOpen') !== 'false' : true;
export const sidebarOpen = writable(storedSidebarOpen);

export const toggleSidebar = () => {
    sidebarOpen.update(val => {
        const newVal = !val;
        if (browser) {
            localStorage.setItem('sidebarOpen', String(newVal));
        }
        return newVal;
    });
};

// ========== GUI Store ==========
export const guiStore=writable({locale:"en-US",os:"Windows_NT",modalShow:false,modalComponent:undefined})
export const showModal=(component)=>{
    guiStore.update(val=>{
        val={...val,modalShow:true,modalComponent:component}
        return val;
    })
}
export const closeModal=()=>{
    guiStore.update(val=>{
        val={...val,modalShow:false,modalComponent:undefined}
        return val
    })
}
export const dataStore=writable({mode:"file",source:[],currentIdx:0});
export const updateData=(obj)=>{
    dataStore.update(val=>{
        val={...val,...obj};
        return val;
    })
}
export const imageStore=writable({scaleX:1.0,scaleY:1.0,rotation:0,pointX:0,pointY:0,exif:undefined});
export const updateImage=(obj)=>{
    imageStore.update(val=>{
        val={...val,...obj};
        return val;
    })
}
export const clearImage=()=>{
    imageStore.update(val=>{
    val.scaleX=1.0;
    val.scaleY=1.0;
    val.rotation=0;
    val.pointX=0;
    val.pointY=0;
    val.exif=undefined;
    return val;
});
    dataStore.update(val=>{
        val.source=[]
        val.currentIdx=0
        val.mode="file"
        return val;
    })
}
export const resetRotation=()=>{
    imageStore.update(val=>{
        val.rotation=0;
        return val;
    })
}
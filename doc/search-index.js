var searchIndex = new Map(JSON.parse('[\
["lib",{"doc":"<code>SsoString</code> in Rust","t":"FPPFTFPPFFUIIGGNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNQNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNQNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNQNNNNNNNNNNNNNNNNNNNNNNNNNCNNFFKFKFNNNNNNNNNNNNNNNMNNNNNMNNMNMNNNNNNNNNNNNNNNNNNN","n":["InvalidArgumentError","Long","Long","LongString","MAX_CAPACITY","RawBuf","Short","Short","ShortString64","SsoStr","SsoString","Str","String","TaggedSsoString64","TaggedSsoString64Mut","add","add_assign","as_bytes","as_bytes","as_bytes","as_mut_str","as_mut_str","as_mut_str","as_mut_vec","as_ptr","as_str","as_str","as_str","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","buf","capacity","capacity","capacity","clear","clone","clone","clone","clone_into","clone_into","clone_into","clone_with_additional_capacity","dangling","dealloc","deref","drain","drop","duck_impl","eq","eq","extend_from_within","fmt","fmt","fmt","fmt","fmt","fmt","fmt","free","from","from","from","from","from","from","from","from","from_raw_parts","from_raw_parts","from_str","from_utf16","from_utf16_lossy","from_utf8","from_utf8_lossy","from_utf8_unchecked","get_non_null","get_non_null_slice","get_non_null_unchecked","get_sized_buf","get_sized_buf","get_sized_buf_mut","insert","insert_str","into","into","into","into","into","into","into","into_boxed_str","into_long","is_long","is_short","is_short","leak","len","len","len","never_impl","new","new","new","next_ptr","next_ptr","pop","push","push","push","push_str","push_str","push_str","push_str_unchecked","push_str_unchecked","realloc","remaining_capacity","remaining_capacity","remove","replace_range","reserve","reserve_exact","retain","set_len","set_len","shrink_to","shrink_to_fit","split_off","tagged","tagged_mut","to_owned","to_owned","to_owned","to_owned","to_string","to_string","to_string","todo_impl","truncate","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_reserve","try_reserve_exact","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","unsafe_field","with_capacity","with_capacity","DeferredSimultaneousUnsafeAssignment","DeferredUnsafeAssignment","SimultaneousUnsafeAssign","SimultaneousUnsafeAssignment","UnsafeAssign","UnsafeField","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","clone","clone_into","from","from","from","from","get","get_mut","get_mut","into","into","into","into","new","new","own","set","set","set_all","set_all","set_all","set_all","to_owned","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","with","with","with"],"q":[[0,"lib"],[180,"lib::unsafe_field"],[232,"alloc::vec"],[233,"core::clone"],[234,"core::result"],[235,"alloc::string"],[236,"core::ops::range"],[237,"core::fmt"],[238,"core::fmt"],[239,"alloc::string"],[240,"core::option"],[241,"alloc::boxed"],[242,"core::ops::function"],[243,"alloc::string"],[244,"core::any"]],"d":["","","","","","","","","","A wrapper around <code>str</code>, so that we can implement <code>ToOwned</code> …","","","","","","","","alias for <code>self.as_str().as_bytes()</code>","Returns a slice of bytes of this string’s contents","Returns a slice of bytes that is always valid utf-8","","","interpret this string as a <code>&amp;str</code>","","Returns the head of this buffer as a raw pointer, this …","interpret this as a <code>&amp;str</code>","","interpret this string as a <code>&amp;str</code>","","","","","","","","","","","","","","","","","","Gets the underyling buffer being used for this string","Returns the capacity of this string, that is, how many …","","Returns the capacity of this short string, this is a …","","","","","","","","clones this string, with at least <code>additional_capacity</code> …","","Deallocates the buffer. Returns <code>InvalidArgumentError</code> if <code>len</code>…","","","","","","","","","","","","","","","free the buffer of this string, setting the <code>len</code> and …","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Construct a new <code>LongString</code> from a <code>length</code>, <code>buf</code> and <code>capacity</code>","Creates a new <code>SsoString::Long</code> from a length, capacity and …","","","","","","","returns a pointer to the element of the buffer that is at …","","get unchecked <code>NonNull&lt;u8&gt;</code> to an index in the buffer, use …","Returns a sized buffer representing the whole buffer of …","Returns <code>buf[0..len]</code> as a <code>NonNull&lt;[u8]&gt;</code> with len <code>len</code>, you …","Returns <code>buf[0..len]</code> as a <code>NonNull&lt;[u8]&gt;</code> with len <code>len</code>, you …","","","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","","Converts this to a <code>LongString</code>. Where the capacity is equal …","Returns <code>!self.is_short()</code>","Returns <code>true</code> if this string is a short string (no heap …","in a union with a long string, returns <code>true</code> if this has …","","returns the length of this string in bytes, length upholds …","","Returns the length of this short string, <code>len</code> upholds fewer …","","","","Constructs and empty ShortString64","returns a pointer to the next element of the buffer that …","Returns the next pointer where we should allocate our …","","Push a <code>char</code> to this string, allocating if needed. Like …","","","Push a <code>str</code> to this string, allocating if needed. Note that …","Push a str <code>s</code> onto the end of this string","","Safety","Safety","realloc to fit at least <code>remaining_capacity</code> more bytes","returns the remaining capacity of this string (how many …","","","","","This doesn’t actually reserve exactly <code>additional</code> extra …","Retains only the characters specified by the predicate.","","Although not unsafe, sa the string is zeroed, you shold …","","This is currently just an alias for …","","Returns the underlying union as an enum, allowing you to …","Same as <code>SsoString::tagged</code>, but returns allows mutation of …","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Construct a new <code>LongString</code> with at least <code>capacity</code> as the …","","","","","Assign several <code>UnsafeField</code> ‘simultaneously’.","","Indicates that a field is unsafe to write to, since we …","","","","","","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Return a reference to the underlying value","Gets a raw pointer to the value","","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Constructs a new <code>UnsafeField</code>","","","Sets the underyling value to <code>value</code>","","Complete all assignments that have been deferred ‘…","","","","","","","","","","","","","","","","","","",""],"i":[0,33,32,0,7,0,33,32,0,0,0,0,0,0,0,1,1,4,1,7,4,1,7,1,9,4,1,7,4,33,32,1,1,25,9,13,7,4,33,32,1,25,9,13,7,4,4,1,7,1,4,9,7,4,9,7,4,9,9,1,1,1,0,1,1,1,4,4,1,1,13,7,7,4,4,33,32,1,1,9,13,7,4,1,4,1,1,1,1,1,4,4,4,4,7,7,1,1,4,33,32,1,9,13,7,1,7,1,1,7,1,4,1,7,0,1,9,7,4,7,1,4,1,7,4,1,7,4,7,4,4,7,1,1,1,1,1,1,7,1,1,1,1,1,4,10,9,7,4,1,7,0,1,4,33,32,1,9,13,7,4,33,32,1,9,13,7,1,1,4,33,32,1,25,9,13,7,0,4,1,0,0,0,0,0,0,40,41,42,37,40,41,42,37,37,37,40,41,42,37,37,38,37,40,41,42,37,38,37,37,38,37,39,40,41,42,37,40,41,42,37,40,41,42,37,40,41,42,37,40,41,42],"f":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,[[1,2],-1,[]],[[1,2],3],[4,[[6,[5]]]],[1,[[6,[5]]]],[7,[[6,[5]]]],[4,2],[1,2],[7,2],[1,[[8,[5]]]],[[[9,[-1]]],[],[]],[4,2],[1,2],[7,2],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[1,10],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[4,[[9,[5]]]],[4,11],[1,11],[7,11],[1,3],[4,4],[[[9,[-1]]],[[9,[-1]]],12],[7,7],[[-1,-2],3,[],[]],[[-1,-2],3,[],[]],[[-1,-2],3,[],[]],[[4,11],4],[[],[[9,[-1]]],[]],[[[9,[-1]],11],[[14,[[9,[-1]],13]]],[]],[1,-1,[]],[[1,-1],15,[[16,[11]]]],[1,3],0,[[1,1],17],[[1,2],17],[[1,-1],3,[[16,[11]]]],[[4,18],19],[[4,18],19],[[1,18],19],[[1,18],19],[[13,18],19],[[7,18],19],[[7,18],19],[4,3],[-1,-1,[]],[-1,-1,[]],[-1,-1,[]],[-1,-1,[]],[2,1],[-1,-1,[]],[-1,-1,[]],[-1,-1,[]],[[[20,[5]],11,11],4],[[5,11,11],1],[2,4],[[[6,[21]]],[[14,[1,22]]]],[[[6,[21]]],1],[[[8,[5]]],[[14,[23,24]]]],[[[6,[5]]],[[26,[25]]]],[[[6,[5]]],1],[[4,11],[[27,[[20,[5]]]]]],[[4,11,11],[[27,[[20,[[6,[5]]]]]]]],[[4,11],[[20,[5]]]],[4,[[20,[[6,[5]]]]]],[7,[[20,[[6,[5]]]]]],[7,[[20,[[6,[5]]]]]],[[1,11,28],3],[[1,11,2],3],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[1,[[29,[2]]]],[[7,11],4],[1,17],[1,17],[7,17],[1,2],[4,11],[1,11],[7,11],0,[[],1],[11,[[3,[[9,[-1]],11]]],[]],[[],7],[4,[[20,[5]]]],[7,[[20,[5]]]],[1,[[27,[28]]]],[[4,28],3],[[1,28],3],[[7,28],3],[[4,2],3],[[1,2],3],[[7,2],3],[[4,2],3],[[7,2],3],[[4,11],3],[4,11],[7,11],[[1,11],28],[[1,-1,2],3,[[16,[11]]]],[[1,11],3],[[1,11],3],[[1,-1],3,[[31,[28],[[30,[17]]]]]],[[1,11],3],[[7,11],3],[[1,11],3],[1,3],[[1,11],23],[1,32],[1,33],[-1,-2,[],[]],[10,-1,[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,34,[]],[-1,34,[]],[-1,34,[]],0,[[1,11],3],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[[1,11],[[14,[3,35]]]],[[1,11],[[14,[3,35]]]],[-1,36,[]],[-1,36,[]],[-1,36,[]],[-1,36,[]],[-1,36,[]],[-1,36,[]],[-1,36,[]],[-1,36,[]],0,[11,4],[11,1],0,0,0,0,0,0,[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[[[37,[-1]]],[[37,[-1]]],12],[[-1,-2],3,[],[]],[-1,-1,[]],[-1,-1,[]],[-1,-1,[]],[-1,-1,[]],[[[37,[-1]]],-1,[]],[38,[[20,[-1]]],[]],[[[37,[-1]]],[[20,[-1]]],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,-2,[],[]],[-1,38,[]],[-1,[[37,[-1]]],[]],[[[37,[-1]]],-1,[]],[[38,-1],3,[]],[[[37,[-1]],-1],3,[]],[39,3],[40,3],[[[41,[-1,-2]]],3,39,39],[[[42,[-2,-1]]],3,[],[[38,[-1]]]],[-1,-2,[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,[[14,[-2]]],[],[]],[-1,36,[]],[-1,36,[]],[-1,36,[]],[-1,36,[]],[[40,-1,-2],[[41,[40,[42,[-2,-1]]]]],[],[[38,[-1]]]],[[[41,[-1,-2]],-3,-4],[[41,[[41,[-1,-2]],[42,[-4,-3]]]]],39,39,[],[[38,[-3]]]],[[[42,[-2,-1]],-3,-4],[[41,[[42,[-2,-1]],[42,[-4,-3]]]]],[],[[38,[-1]]],[],[[38,[-3]]]]],"c":[],"p":[[20,"SsoString",0],[1,"str"],[1,"tuple"],[5,"LongString",0],[1,"u8"],[1,"slice"],[5,"ShortString64",0],[5,"Vec",232],[5,"RawBuf",0],[8,"Str",0],[1,"usize"],[10,"Clone",233],[5,"InvalidArgumentError",0],[6,"Result",234],[5,"Drain",235],[10,"RangeBounds",236],[1,"bool"],[5,"Formatter",237],[8,"Result",237],[5,"NonNull",238],[1,"u16"],[5,"FromUtf16Error",235],[8,"String",0],[5,"FromUtf8Error",235],[5,"SsoStr",0],[6,"Cow",239],[6,"Option",240],[1,"char"],[5,"Box",241],[17,"Output"],[10,"FnMut",242],[6,"TaggedSsoString64",0],[6,"TaggedSsoString64Mut",0],[5,"String",235],[5,"TryReserveError",243],[5,"TypeId",244],[5,"UnsafeField",180],[10,"UnsafeAssign",180],[10,"SimultaneousUnsafeAssign",180],[5,"SimultaneousUnsafeAssignment",180],[5,"DeferredSimultaneousUnsafeAssignment",180],[5,"DeferredUnsafeAssignment",180]],"b":[[63,"impl-PartialEq-for-SsoString"],[64,"impl-PartialEq%3Cstr%3E-for-SsoString"],[66,"impl-Display-for-LongString"],[67,"impl-Debug-for-LongString"],[68,"impl-Display-for-SsoString"],[69,"impl-Debug-for-SsoString"],[71,"impl-Display-for-ShortString64"],[72,"impl-Debug-for-ShortString64"]]}]\
]'));
if (typeof exports !== 'undefined') exports.searchIndex = searchIndex;
else if (window.initSearch) window.initSearch(searchIndex);
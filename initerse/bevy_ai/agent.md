Always use MessageReader instead of EventReader, same for EventWriter and MessageWriter (which then the function isn't send but write)
On querys, you can use get_single or get_single_mut but instead use single() and single_mut()
